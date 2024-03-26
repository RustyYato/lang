use crate::{
    ctx::{AllocContext, ContextId},
    ptr::ContextPtr,
};

pub struct TypeHeader {
    kind: TypeKind,
}

impl TypeHeader {
    pub const fn new(kind: TypeKind) -> Self {
        Self { kind }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeKind {
    Unit,
}

pub unsafe trait BasicTypeData<'ctx>:
    TypeData<'ctx, Target = Self> + init::Ctor<Self::InitArgs>
{
    type InitArgs;
    type LayoutProvider: init::layout_provider::LayoutProvider<Self, Self::InitArgs>;

    const KIND: TypeKind;
}

unsafe impl<'ctx, T: BasicTypeData<'ctx>> TypeData<'ctx> for T {
    type Target = Self;

    fn try_cast(ptr: Type<'ctx>) -> Option<Type<'ctx, Self>> {
        if ptr.kind() == T::KIND {
            unsafe { Some(Type::from_raw(ptr.0.id(), ptr.into_raw().cast())) }
        } else {
            None
        }
    }
}

pub unsafe trait TypeData<'ctx>: 'ctx + Send + Sync {
    type Target: ?Sized;

    fn try_cast(ptr: Type<'ctx>) -> Option<Type<'ctx, Self::Target>>;
}

unsafe impl<'ctx, T: ?Sized + TypeData<'ctx>> TypeData<'ctx> for Type<'ctx, T> {
    type Target = T::Target;

    fn try_cast(ptr: Type<'ctx>) -> Option<Type<'ctx, Self::Target>> {
        ptr.try_cast::<T>()
    }
}

pub struct Type<'ctx, T: ?Sized = TypeHeader>(ContextPtr<'ctx, T>);

impl<T: ?Sized> Copy for Type<'_, T> {}
impl<T: ?Sized> Clone for Type<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'ctx, T: ?Sized + BasicTypeData<'ctx>> Type<'ctx, T> {
    pub const fn kind(self) -> TypeKind {
        T::KIND
    }
}

impl<'ctx, T: ?Sized> Type<'ctx, T> {
    pub const fn erase(self) -> Type<'ctx> {
        Type(unsafe { self.0.cast() })
    }

    pub const fn into_raw(self) -> *const T {
        self.0.as_ptr()
    }

    pub const unsafe fn from_raw(id: ContextId<'ctx>, ptr: *const T) -> Self {
        Self(ContextPtr::new_unchecked(id, ptr))
    }

    pub const fn id(self) -> ContextId<'ctx> {
        self.0.id()
    }

    pub const fn header(self) -> &'ctx TypeHeader {
        self.erase().0.as_ref()
    }
}

impl TypeHeader {
    pub const fn kind(&self) -> TypeKind {
        self.kind
    }
}

impl<'ctx> Type<'ctx> {
    pub fn try_cast<T: ?Sized + TypeData<'ctx>>(self) -> Option<Type<'ctx, T::Target>> {
        T::try_cast(self)
    }

    pub const fn kind(self) -> TypeKind {
        self.0.as_ref().kind
    }
}

impl<'ctx, T: ?Sized + BasicTypeData<'ctx>> init::Ctor<(T::InitArgs, AllocContext<'ctx>)>
    for Type<'ctx, T>
{
    type Error = T::Error;

    fn try_init<'a>(
        ptr: init::ptr::Uninit<'a, Self>,
        (args, alloc): (T::InitArgs, AllocContext<'ctx>),
    ) -> Result<init::ptr::Init<'a, Self>, Self::Error> {
        let ctx_ptr = alloc.try_init::<T, T::InitArgs, T::LayoutProvider>(args)?;
        Ok(ptr.write(Self(ctx_ptr)))
    }
}