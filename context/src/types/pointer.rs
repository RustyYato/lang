use super::raw::{BasicTypeData, RawType, TypeHeader, TypeKind};

pub type PointerTy<'ctx> = RawType<'ctx, PointerData>;

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
