use super::raw::{BasicTypeData, RawType, TypeHeader, TypeKind};

pub type UnitTy<'ctx> = RawType<'ctx, UnitData>;

#[repr(C)]
pub struct UnitData {
    header: TypeHeader,
}

impl init::Ctor for UnitData {
    type Error = core::convert::Infallible;

    fn try_init(
        ptr: init::ptr::Uninit<Self>,
        (): (),
    ) -> Result<init::ptr::Init<Self>, Self::Error> {
        Ok(ptr.write(Self {
            header: TypeHeader::of::<Self>(),
        }))
    }
}

unsafe impl<'ctx> BasicTypeData<'ctx> for UnitData {
    const KIND: TypeKind = TypeKind::Unit;

    fn layout(&self, _ctx: crate::Context<'ctx>) -> super::raw::Layout {
        super::raw::Layout::Concrete(super::raw::ConcreteLayout { size: 0, align: 1 })
    }
}
