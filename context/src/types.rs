mod raw;

mod float;
mod int;
mod unit;

pub type Type<'ctx> = raw::Type<'ctx>;
pub use raw::TypeKind;

pub use float::{FloatKind, FloatTy};
pub use int::IntTy;
pub use unit::UnitTy;
