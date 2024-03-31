use std::hash::Hash;

use crate::{
    ctx::{AllocContext, ContextId},
    ptr::ContextPtr,
};

pub struct TypeHeader {
    kind: TypeKind,
}

impl TypeHeader {
    pub const fn of<'ctx, T: ?Sized + BasicTypeData<'ctx>>() -> Self {
        Self { kind: T::KIND }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeKind {
    Unit,
    Int,
    Float,
    Pointer,
    Aggregate,
    Func,
}

pub unsafe trait BasicTypeData<'ctx>: 'ctx {
    const KIND: TypeKind;
}

unsafe impl<'ctx, T: BasicTypeData<'ctx>> TypeData<'ctx> for T {
    type Target = Self;

    fn try_cast(ptr: RawType<'ctx>) -> Option<RawType<'ctx, Self>> {
        if ptr.kind() == T::KIND {
            unsafe { Some(RawType::from_raw(ptr.0.id(), ptr.into_raw().cast())) }
        } else {
            None
        }
    }
}

pub unsafe trait TypeData<'ctx>: 'ctx {
    type Target: ?Sized;

    fn try_cast(ptr: RawType<'ctx>) -> Option<RawType<'ctx, Self::Target>>;
}

unsafe impl<'ctx, T: ?Sized + TypeData<'ctx>> TypeData<'ctx> for RawType<'ctx, T> {
    type Target = T::Target;

    fn try_cast(ptr: RawType<'ctx>) -> Option<RawType<'ctx, Self::Target>> {
        ptr.try_cast::<T>()
    }
}

/// NOTE: This type is only visible in documentation so you can see methods on it's aliases
/// It will not actually be available while programming except through those aliases
pub struct RawType<'ctx, T: ?Sized = TypeHeader>(ContextPtr<'ctx, T>);

impl<T: ?Sized> Copy for RawType<'_, T> {}
impl<T: ?Sized> Clone for RawType<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Eq for RawType<'_, T> {}
impl<T: ?Sized> PartialEq for RawType<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<T: ?Sized> Hash for RawType<'_, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<'ctx, T: ?Sized + BasicTypeData<'ctx>> RawType<'ctx, T> {
    pub const fn kind(self) -> TypeKind {
        T::KIND
    }
}

impl<'ctx, T: ?Sized> RawType<'ctx, T> {
    pub const fn erase(self) -> super::Type<'ctx> {
        RawType(unsafe { self.0.cast() })
    }

    #[doc(hidden)]
    pub const fn into_raw(self) -> *const T {
        self.0.as_ptr()
    }

    #[doc(hidden)]
    pub const unsafe fn from_raw(id: ContextId<'ctx>, ptr: *const T) -> Self {
        Self(ContextPtr::new_unchecked(id, ptr))
    }

    pub const fn id(self) -> ContextId<'ctx> {
        self.0.id()
    }

    #[doc(hidden)]
    pub const fn header(self) -> &'ctx TypeHeader {
        self.erase().get()
    }

    pub const fn get(self) -> &'ctx T {
        self.0.as_ref()
    }
}

impl TypeHeader {
    pub const fn kind(&self) -> TypeKind {
        self.kind
    }
}

impl<'ctx> super::Type<'ctx> {
    pub fn try_cast<T: ?Sized + TypeData<'ctx>>(self) -> Option<RawType<'ctx, T::Target>> {
        T::try_cast(self)
    }

    pub fn cast<T: ?Sized + TypeData<'ctx>>(self) -> RawType<'ctx, T::Target> {
        fn bad_cast<Target: ?Sized>(kind: TypeKind) -> ! {
            panic!(
                "Could not cast {kind:?} to {}",
                core::any::type_name::<Target>()
            )
        }

        match T::try_cast(self) {
            Some(ptr) => ptr,
            None => bad_cast::<T::Target>(self.kind()),
        }
    }

    pub const fn kind(self) -> TypeKind {
        self.0.as_ref().kind
    }
}

impl<'ctx, T: ?Sized> RawType<'ctx, T> {
    pub(crate) fn init<A>(
        args: A,
        alloc: AllocContext<'ctx>,
    ) -> impl init::Initializer<Self, Error = A::Error>
    where
        A: init::Initializer<T>,
        T: Sized,
    {
        Self::init_with::<A, init::layout_provider::SizedLayout>(args, alloc)
    }

    pub fn init_with<A, L>(
        args: A,
        alloc: AllocContext<'ctx>,
    ) -> impl init::Initializer<Self, Error = A::Error>
    where
        A: init::Initializer<T>,
        L: init::layout_provider::LayoutProvider<T, A>,
    {
        init::try_init_fn(move |ptr| {
            let ctx_ptr = alloc.try_init::<T, A, L>(args)?;
            Ok(ptr.write(Self(ctx_ptr)))
        })
    }
}
