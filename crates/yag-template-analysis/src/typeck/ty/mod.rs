use std::fmt;

use ecow::EcoVec;

mod display;
pub mod foreign;
mod ops;

pub use display::TyDisplay;
pub use foreign::{
    CallableHandle, CallableTy, MapHandle, MapTy, NewtypeHandle, NewtypeTy, SliceHandle, SliceTy, StructHandle,
    StructTy, TypedStrMapHandle, TypedStrMapTy,
};
pub use ops::{indirect, loosely_assignable, underlying};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    Any,
    Union(EcoVec<Ty>),
    Pointer(Box<Ty>),

    Struct(StructHandle),
    Callable(CallableHandle),
    Newtype(NewtypeHandle),
    Map(MapHandle),
    TypedStrMap(TypedStrMapHandle),
    Slice(SliceHandle),

    Primitive(PrimitiveTy),
}

impl Ty {
    pub fn is_any(&self) -> bool {
        self == &Ty::Any
    }

    pub fn is_primitive(&self) -> bool {
        matches!(self, Ty::Primitive(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveTy {
    String,
    Bool,
    Int,
    Int64,
    Float64,
    Byte,
    Rune,
    Nil,
}

impl fmt::Display for PrimitiveTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use PrimitiveTy::*;
        f.write_str(match self {
            String => "string",
            Bool => "bool",
            Int => "int",
            Int64 => "int64",
            Float64 => "float64",
            Byte => "byte",
            Rune => "rune",
            Nil => "nil",
        })
    }
}

impl PrimitiveTy {
    pub fn class(self) -> PrimitiveClass {
        use PrimitiveTy::*;
        match self {
            String => PrimitiveClass::String,
            Bool => PrimitiveClass::Bool,
            Int | Int64 | Byte | Rune => PrimitiveClass::Integer,
            Float64 => PrimitiveClass::FloatingPoint,
            Nil => PrimitiveClass::Nil,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveClass {
    String,
    Bool,
    Integer,
    FloatingPoint,
    Nil,
}
