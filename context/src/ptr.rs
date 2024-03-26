use std::{marker::PhantomData, ptr::NonNull};

use crate::ctx::ContextId;

pub(crate) struct ContextPtr<'ctx, T: ?Sized + 'ctx>(NonNull<T>, ContextId<'ctx>);

unsafe impl<T: Sync> Send for ContextPtr<'_, T> {}
unsafe impl<T: Sync> Sync for ContextPtr<'_, T> {}

impl<T: ?Sized> Copy for ContextPtr<'_, T> {}
impl<T: ?Sized> Clone for ContextPtr<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'ctx, T: ?Sized> ContextPtr<'ctx, T> {
    pub fn from_ref(id: ContextId<'ctx>, ptr: &'ctx T) -> Self {
        Self(NonNull::from(ptr), id)
    }

    pub const unsafe fn new_unchecked(id: ContextId<'ctx>, ptr: *const T) -> Self {
        Self(NonNull::new_unchecked(ptr.cast_mut()), id)
    }

    pub const fn as_ptr(self) -> *const T {
        self.0.as_ptr()
    }

    pub fn id(self) -> ContextId<'ctx> {
        self.1
    }
}
