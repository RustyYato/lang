use std::{marker::PhantomData, num::NonZeroU16};

use crate::{ptr::ContextPtr, TargetSpec};

#[derive(Clone, Copy)]
pub struct ContextId<'ctx>(PhantomData<fn() -> *mut &'ctx mut ()>);

mod ty;

struct ContextData<'ctx> {
    id: ContextId<'ctx>,
    target: TargetSpec,
    ty: ty::TypeContextData<'ctx>,
    // put Bump last so if anything tries to access it on drop, the data will not be
    // used after bump is dropped. Thus hardenning against use after frees.
    // We currently cannot guarantee this on stable, because #[may_dangle] is not stable
    // so hashbrown doesn't implement it on stable.
    bump: bumpme::Bump,
}

#[derive(Clone, Copy)]
pub struct Context<'ctx>(ContextPtr<'ctx, ContextData<'ctx>>);

#[derive(Clone, Copy)]
pub struct TypeContext<'ctx>(ContextPtr<'ctx, ty::TypeContextData<'ctx>>);

#[derive(Clone, Copy)]
pub struct AllocContext<'ctx>(ContextPtr<'ctx, bumpme::Bump>);

impl<'ctx> Context<'ctx> {
    pub fn with<T>(target: TargetSpec, f: impl FnOnce(Context<'_>) -> T) -> T {
        let ctx_data: ContextData =
            init::try_init_on_stack(target).unwrap_or_else(|inf| match inf {});
        let ctx = Context(unsafe {
            ContextPtr::new_unchecked(ctx_data.id, core::ptr::addr_of!(ctx_data))
        });
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
        let ptr = self.0.as_ref();
        AllocContext(ContextPtr::from_ref(self.id(), &ptr.bump))
    }

    #[inline]
    pub const fn unit_ty(self) -> crate::types::UnitTy<'ctx> {
        self.type_ctx().unit()
    }

    #[inline]
    pub const fn pointer_ty(self) -> crate::types::PointerTy<'ctx> {
        self.type_ctx().pointer()
    }

    #[inline]
    pub fn int_ty(self, bits: u16) -> crate::types::IntTy<'ctx> {
        self.type_ctx().int(
            self.alloc_ctx(),
            NonZeroU16::new(bits).expect("cannot construct a zero-sized int type"),
        )
    }

    #[inline]
    pub const fn float_16_ty(self) -> crate::types::FloatTy<'ctx> {
        self.type_ctx().float(crate::types::FloatKind::Ieee16Bit)
    }

    #[inline]
    pub const fn float_32_ty(self) -> crate::types::FloatTy<'ctx> {
        self.type_ctx().float(crate::types::FloatKind::Ieee32Bit)
    }

    #[inline]
    pub const fn float_64_ty(self) -> crate::types::FloatTy<'ctx> {
        self.type_ctx().float(crate::types::FloatKind::Ieee64Bit)
    }

    #[inline]
    pub const fn float_128_ty(self) -> crate::types::FloatTy<'ctx> {
        self.type_ctx().float(crate::types::FloatKind::Ieee128Bit)
    }
}

impl<'ctx> AllocContext<'ctx> {
    #[inline]
    pub const fn id(self) -> ContextId<'ctx> {
        ContextId(PhantomData)
    }

    pub(crate) fn try_init<T, Args, L>(self, args: Args) -> Result<ContextPtr<'ctx, T>, Args::Error>
    where
        T: ?Sized,
        Args: init::Initializer<T>,
        L: init::layout_provider::LayoutProvider<T, Args>,
    {
        let layout = L::layout_for(&args).expect("could not construct layout");
        let bump = unsafe { &*self.0.as_ptr() };
        let ptr = bump.alloc_layout(layout).into_raw();
        let ptr = unsafe { L::cast(ptr, &args) };
        let ptr = unsafe { init::ptr::Uninit::from_raw(ptr.as_ptr()) };
        let ptr = ptr.try_init(args)?.into_raw();
        Ok(unsafe { ContextPtr::new_unchecked(self.id(), ptr) })
    }
}

impl<'ctx> init::Ctor<TargetSpec> for ContextData<'ctx> {
    type Error = core::convert::Infallible;

    fn try_init<'a>(
        ptr: init::ptr::Uninit<'a, Self>,
        spec: TargetSpec,
    ) -> Result<init::ptr::Init<'a, Self>, Self::Error> {
        init::init_struct! {
            ptr => Self {
                id: init::init_fn(|ptr| ptr.write( ContextId(PhantomData))),
                target: init::init(spec),
                bump: init::init(bumpme::Bump::new()),
                ty: ty::TypeContextDataArgs {
                    alloc:  AllocContext(unsafe { ContextPtr::new_unchecked(*id, bump.as_ptr()) }),
                    target: &target,
                }
            }
        }
    }
}
