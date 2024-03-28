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
            header: TypeHeader::of::<Self>(),
            kind,
        }))
    }
}

unsafe impl<'ctx> BasicTypeData<'ctx> for FloatData {
    const KIND: TypeKind = TypeKind::Float;
}
