use crate::types;

use super::AllocContext;

pub(super) struct TypeContextData<'ctx> {
    pub unit: types::UnitTy<'ctx>,
}

pub(super) struct TypeContextDataArgs<'ctx> {
    pub alloc: AllocContext<'ctx>,
}

impl<'ctx> init::Ctor<TypeContextDataArgs<'ctx>> for TypeContextData<'ctx> {
    type Error = core::convert::Infallible;

    fn try_init<'a>(
        ptr: init::ptr::Uninit<'a, Self>,
        args: TypeContextDataArgs<'ctx>,
    ) -> Result<init::ptr::Init<'a, Self>, Self::Error> {
        init::init_struct! {
            ptr => Self {
                unit: ((), args.alloc)
            }
        }
    }
}
