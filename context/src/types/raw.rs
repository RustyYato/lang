use crate::{
    ctx::{AllocContext, ContextId},
    ptr::ContextPtr,
};

pub struct TypeHeader<'ctx> {
    id: ContextId<'ctx>,
    kind: TypeKind,
}

impl<'ctx> TypeHeader<'ctx> {
    pub const fn new(id: ContextId<'ctx>, kind: TypeKind) -> Self {
        Self { id, kind }
    }
}

#[derive(Clone, Copy)]
pub enum TypeKind {
    //
}

pub unsafe trait BasicTypeData<'ctx>: TypeData<'ctx> + init::Ctor<Self::InitArgs> {
    type InitArgs;

    const KIND: TypeKind;
}

pub unsafe trait TypeData<'ctx>: 'ctx + Send + Sync {
    fn unerase(ptr: Type<'ctx>) -> Type<'ctx, Self>;
}

pub struct Type<'ctx, T: ?Sized = TypeHeader<'ctx>>(ContextPtr<'ctx, T>);

impl<'ctx, T: ?Sized + BasicTypeData<'ctx>> Type<'ctx, T> {
    pub(crate) fn new<L>(args: T::InitArgs, alloc: AllocContext<'ctx>) -> Self
    where
        T::Error: core::fmt::Debug,
        L: init::layout_provider::LayoutProvider<T, T::InitArgs>,
    {
        Type(alloc.init::<T, T::InitArgs, L>(args))
    }

    pub const fn kind(self) -> TypeKind {
        T::KIND
    }
}

impl<'ctx, T: ?Sized> Type<'ctx, T> {
    pub const fn erase(self) -> Type<'ctx> {
        Type(unsafe { self.0.cast() })
    }

    pub const fn header(self) -> &'ctx TypeHeader<'ctx> {
        self.erase().0.as_ref()
    }
}

impl<'ctx> TypeHeader<'ctx> {
    pub const fn id(&'ctx self) -> ContextId<'ctx> {
        self.id
    }

    pub const fn kind(&'ctx self) -> TypeKind {
        self.kind
    }
}

impl<'ctx> Type<'ctx> {
    pub fn unerase<T: TypeData<'ctx>>(self) -> Type<'ctx, T> {
        T::unerase(self)
    }

    pub const fn kind(self) -> TypeKind {
        self.0.as_ref().kind
    }
}
