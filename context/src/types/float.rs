use super::raw::{BasicTypeData, RawType, TypeHeader, TypeKind};

pub type FloatTy<'ctx> = RawType<'ctx, FloatData>;

#[repr(C)]
pub struct FloatData {
    header: TypeHeader,
    pub kind: FloatKind,
}

#[derive(Debug, Clone, Copy)]
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

    fn layout(&self, _ctx: crate::Context<'ctx>) -> super::raw::Layout {
        match self.kind {
            FloatKind::Ieee16Bit => {
                super::raw::Layout::Concrete(super::raw::ConcreteLayout { size: 2, align: 2 })
            }
            FloatKind::Ieee32Bit => {
                super::raw::Layout::Concrete(super::raw::ConcreteLayout { size: 4, align: 4 })
            }
            FloatKind::Ieee64Bit => {
                super::raw::Layout::Concrete(super::raw::ConcreteLayout { size: 8, align: 8 })
            }
            FloatKind::Ieee128Bit => super::raw::Layout::Concrete(super::raw::ConcreteLayout {
                size: 16,
                align: 16,
            }),
        }
    }
}

impl FloatTy<'_> {
    pub const fn float_kind(self) -> FloatKind {
        self.get().kind
    }
}
