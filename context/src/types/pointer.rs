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

    fn layout(&self, ctx: crate::Context<'ctx>) -> super::raw::Layout {
        let size = ctx.target().pointer_size_bytes as u64;
        let align = 1 << ctx.target().pointer_align_log2;
        super::raw::Layout::Concrete(super::raw::ConcreteLayout { size, align })
    }
}
