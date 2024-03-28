use super::raw::{BasicTypeData, Type, TypeHeader, TypeKind};

pub type PointerTy<'ctx> = Type<'ctx, PointerData>;

#[repr(C)]
pub struct PointerData {
    header: TypeHeader,
}

impl init::Ctor for PointerData {
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

unsafe impl<'ctx> BasicTypeData<'ctx> for PointerData {
    const KIND: TypeKind = TypeKind::Pointer;
}
