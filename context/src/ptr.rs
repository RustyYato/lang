use std::{hash::Hash, ptr::NonNull};

use crate::ctx::ContextId;

pub(crate) struct ContextPtr<'ctx, T: ?Sized + 'ctx>(NonNull<T>, ContextId<'ctx>);

impl<T: ?Sized> Copy for ContextPtr<'_, T> {}
impl<T: ?Sized> Clone for ContextPtr<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Eq for ContextPtr<'_, T> {}
impl<T: ?Sized> PartialEq for ContextPtr<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: ?Sized> Hash for ContextPtr<'_, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<'ctx, T: ?Sized> ContextPtr<'ctx, T> {
    pub const fn from_ref(id: ContextId<'ctx>, ptr: &'ctx T) -> Self {
        Self(
            unsafe { NonNull::new_unchecked(ptr as *const T as *mut T) },
            id,
        )
    }

    pub const unsafe fn new_unchecked(id: ContextId<'ctx>, ptr: *const T) -> Self {
        Self(NonNull::new_unchecked(ptr.cast_mut()), id)
    }

    pub const fn as_ptr(self) -> *const T {
        self.0.as_ptr()
    }

    pub const fn as_ref(self) -> &'ctx T {
        unsafe { self.0.as_ref() }
    }

    pub const unsafe fn cast<U>(self) -> ContextPtr<'ctx, U> {
        ContextPtr(self.0.cast(), self.1)
    }

    pub const fn id(self) -> ContextId<'ctx> {
        self.1
    }
}
