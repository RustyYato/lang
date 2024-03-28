mod raw;

mod float;
mod int;
mod pointer;
mod unit;

pub type Type<'ctx> = raw::Type<'ctx>;
pub use raw::TypeKind;

pub use float::{FloatKind, FloatTy};
pub use int::IntTy;
pub use pointer::PointerTy;
pub use unit::UnitTy;
