use super::raw::{BasicTypeData, Type, TypeHeader, TypeKind};

pub type FloatTy<'ctx> = Type<'ctx, FloatData>;

#[repr(C)]
pub struct FloatData {
    header: TypeHeader,
    pub kind: FloatKind,
}

pub enum FloatKind {
    Ieee16Bit,
    Ieee32Bit,
    Ieee64Bit,
    Ieee128Bit,
}

impl init::Ctor<FloatKind> for FloatData {
    type Error = core::convert::Infallible;

    fn try_init(
        ptr: init::ptr::Uninit<Self>,
        kind: FloatKind,
    ) -> Result<init::ptr::Init<Self>, Self::Error> {
        Ok(ptr.write(Self {
            header: TypeHeader::new(TypeKind::Unit),
            kind,
        }))
    }
}

unsafe impl<'ctx> BasicTypeData<'ctx> for FloatData {
    type InitArgs = FloatKind;
    type LayoutProvider = init::layout_provider::SizedLayout;
    const KIND: TypeKind = TypeKind::Float;
}
