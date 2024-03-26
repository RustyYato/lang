use super::{AllocContext, ContextId};

pub(super) struct TypeContextData<'ctx> {
    id: ContextId<'ctx>,
}

pub(super) struct TypeContextDataArgs<'ctx> {
    pub id: ContextId<'ctx>,
    pub bump: AllocContext<'ctx>,
}

impl<'ctx> init::Ctor<TypeContextDataArgs<'ctx>> for TypeContextData<'ctx> {
    type Error = core::convert::Infallible;

    fn try_init<'a>(
        ptr: init::ptr::Uninit<'a, Self>,
        args: TypeContextDataArgs<'ctx>,
    ) -> Result<init::ptr::Init<'a, Self>, Self::Error> {
        init::init_struct! {
            ptr => Self {
                id: init::init_fn(|ptr| ptr.write(args.id))
            }
        }
    }
}
