use std::fmt;

mod display;
pub mod foreign;
mod ops;
mod union;

pub use display::TyDisplay;
pub use foreign::{
    MapHandle, MapTy, MethodHandle, MethodTy, NewtypeHandle, NewtypeTy, SliceHandle, SliceTy, StaticStrMapHandle,
    StaticStrMapTy, StructHandle, StructTy,
};
pub use ops::{base_ty, indirect, loosely_assignable};
pub use union::{union, UnionTy};

/// An immutable, cheaply cloneable type.
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum Ty {
    Any,
    Never,
    Union(UnionTy),
    Pointer(Box<Ty>),

    // Foreign types only accessible through functions provided by the host environment.
    Struct(StructHandle),
    Method(MethodHandle),
    Newtype(NewtypeHandle),
    Map(MapHandle),
    StaticStrMap(StaticStrMapHandle),
    Slice(SliceHandle),

    Primitive(PrimitiveTy),

    // Intrinsic/special types.
    TemplateName,
}

impl Ty {
    pub fn is_any(&self) -> bool {
        self == &Ty::Any
    }

    pub fn is_inhabited(&self) -> bool {
        self != &Ty::Never
    }

    pub fn is_never(&self) -> bool {
        self == &Ty::Never
    }

    pub fn is_primitive(&self) -> bool {
        matches!(self, Ty::Primitive(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
