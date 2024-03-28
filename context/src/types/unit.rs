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
}
