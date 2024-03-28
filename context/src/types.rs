mod raw;

mod aggregate;
mod float;
mod int;
mod pointer;
mod unit;

pub type Type<'ctx> = raw::RawType<'ctx>;
pub use raw::TypeKind;

pub use aggregate::{AggregateField, AggregateLayoutProvider, AggregateTy};
pub use float::{FloatKind, FloatTy};
pub use int::IntTy;
pub use pointer::PointerTy;
pub use unit::UnitTy;
