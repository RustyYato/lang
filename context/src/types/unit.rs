use super::raw::{BasicTypeData, Type, TypeHeader, TypeKind};

pub type UnitTy<'ctx> = Type<'ctx, UnitData>;

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
    type InitArgs = ();
    type LayoutProvider = init::layout_provider::SizedLayout;
    const KIND: TypeKind = TypeKind::Unit;
}
