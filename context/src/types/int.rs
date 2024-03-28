use std::num::NonZeroU16;

use super::raw::{BasicTypeData, Type, TypeHeader, TypeKind};

pub type IntTy<'ctx> = Type<'ctx, IntData>;

#[repr(C)]
pub struct IntData {
    header: TypeHeader,
    pub bits: NonZeroU16,
}

impl init::Ctor<u16> for IntData {
    type Error = core::convert::Infallible;

    fn try_init(
        ptr: init::ptr::Uninit<Self>,
        bits: u16,
    ) -> Result<init::ptr::Init<Self>, Self::Error> {
        Ok(ptr.write(Self {
            header: TypeHeader::of::<Self>(),
            bits: NonZeroU16::new(bits).expect("you must pass a non-zero u16"),
        }))
    }
}

unsafe impl<'ctx> BasicTypeData<'ctx> for IntData {
    type InitArgs = u16;
    type LayoutProvider = init::layout_provider::SizedLayout;
    const KIND: TypeKind = TypeKind::Int;
}
