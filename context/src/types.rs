mod raw;

mod int;
mod unit;

pub type Type<'ctx> = raw::Type<'ctx>;
pub use raw::TypeKind;

pub use int::IntTy;
pub use unit::UnitTy;
