use std::num::NonZeroU16;

use super::raw::{BasicTypeData, RawType, TypeHeader, TypeKind};

pub type IntTy<'ctx> = RawType<'ctx, IntData>;

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
    const KIND: TypeKind = TypeKind::Int;

    fn layout(&self, _ctx: crate::Context<'ctx>) -> super::raw::Layout {
        let size = self.bits.get().div_ceil(8);
        let align = if self.bits.get() % 8 == 0 {
            self.bits.get() / 8
        } else {
            1
        };
        let align = (align / 2 + 1).next_power_of_two();
        let align = crate::utils::gcd(size, align);

        super::raw::Layout::Concrete(super::raw::ConcreteLayout {
            size: size as u64,
            align: align as u64,
        })
    }
}

impl IntTy<'_> {
    pub const fn bits(self) -> NonZeroU16 {
        self.get().bits
    }
}

#[test]
fn test() {
    crate::Context::with(crate::TEST_TARGET_SPEC, |ctx| {
        dbg!(ctx.int_ty(48).cast::<IntTy>().get().layout(ctx));
    })
}
