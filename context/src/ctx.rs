use std::marker::PhantomData;

use crate::ptr::ContextPtr;

#[derive(Clone, Copy)]
pub struct ContextId<'ctx>(PhantomData<fn() -> *mut &'ctx mut ()>);

mod ty;

struct ContextData<'ctx> {
    id: ContextId<'ctx>,
    bump: bumpme::Bump,
    ty: ty::TypeContextData<'ctx>,
}

#[derive(Clone, Copy)]
pub struct Context<'ctx>(ContextPtr<'ctx, ContextData<'ctx>>);

#[derive(Clone, Copy)]
pub struct TypeContext<'ctx>(ContextPtr<'ctx, ty::TypeContextData<'ctx>>);

#[derive(Clone, Copy)]
pub struct AllocContext<'ctx>(ContextPtr<'ctx, bumpme::Bump>);

impl<'ctx> Context<'ctx> {
    pub fn with<T>(f: impl FnOnce(Context<'_>) -> T) -> T {
        let ctx_data: ContextData = init::try_init_on_stack(()).unwrap_or_else(|inf| match inf {});
        let ctx = Context(ContextPtr::from_ref(ctx_data.id, &ctx_data));
        f(ctx)
    }

    #[inline]
    pub const fn id(self) -> ContextId<'ctx> {
        ContextId(PhantomData)
    }

    #[inline]
    pub const fn type_ctx(self) -> TypeContext<'ctx> {
        let ptr = self.0.as_ptr();
        let ptr = unsafe { core::ptr::addr_of!((*ptr).ty) };
        TypeContext(unsafe { ContextPtr::new_unchecked(self.id(), ptr) })
    }

    #[inline]
    pub const fn alloc_ctx(self) -> AllocContext<'ctx> {
        let ptr = self.0.as_ptr();
        let ptr = unsafe { core::ptr::addr_of!((*ptr).bump) };
        AllocContext(unsafe { ContextPtr::new_unchecked(self.id(), ptr) })
    }
}

impl<'ctx> TypeContext<'ctx> {
    pub const fn id(self) -> ContextId<'ctx> {
        ContextId(PhantomData)
    }
}

impl<'ctx> AllocContext<'ctx> {
    #[inline]
    pub const fn id(self) -> ContextId<'ctx> {
        ContextId(PhantomData)
    }

    pub(crate) fn init<T, Args, L>(self, args: Args) -> ContextPtr<'ctx, T>
    where
        T: ?Sized + init::Ctor<Args>,
        T::Error: core::fmt::Debug,
        L: init::layout_provider::LayoutProvider<T, Args>,
    {
        let layout = L::layout_for(&args).expect("could not construct layout");
        let bump = unsafe { &*self.0.as_ptr() };
        let ptr = bump.alloc_layout(layout).into_raw();
        let ptr = unsafe { L::cast(ptr, &args) };
        let ptr = unsafe { init::ptr::Uninit::from_raw(ptr.as_ptr()) };
        let ptr = ptr.try_init(args).unwrap().into_raw();
        unsafe { ContextPtr::new_unchecked(self.id(), ptr) }
    }
}

impl<'ctx> init::Ctor for ContextData<'ctx> {
    type Error = core::convert::Infallible;

    fn try_init(
        ptr: init::ptr::Uninit<Self>,
        (): (),
    ) -> Result<init::ptr::Init<Self>, Self::Error> {
        init::init_struct! {
            ptr => Self {
                id: init::init_fn(|ptr| ptr.write( ContextId(PhantomData))),
                bump: init::init_fn(|ptr| ptr.write(bumpme::Bump::new())),
                ty: ty::TypeContextDataArgs {
                    id: *id,
                    bump:  AllocContext(unsafe { ContextPtr::new_unchecked(*id, bump.as_ptr()) }),
                }
            }
        }
    }
}
