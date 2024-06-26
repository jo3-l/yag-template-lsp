use core::slice;
use std::fmt;

use ecow::EcoVec;

mod display;
pub mod foreign;
mod ops;

pub use display::TyDisplay;
pub use foreign::{
    MapHandle, MapTy, MethodHandle, MethodTy, NewtypeHandle, NewtypeTy, SliceHandle, SliceTy, StructHandle, StructTy,
    TypedStrMapHandle, TypedStrMapTy,
};
pub use ops::{indirect, loosely_assignable, underlying, union};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    Any,
    Never,
    Union(UnionTy),
    Pointer(Box<Ty>),

    Struct(StructHandle),
    Method(MethodHandle),
    Newtype(NewtypeHandle),
    Map(MapHandle),
    TypedStrMap(TypedStrMapHandle),
    Slice(SliceHandle),

    Primitive(PrimitiveTy),

    // Special types.
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

/// A union of multiple types.
///
/// By construction, union types always have two or more constituents. All operations that produce
/// union types: that is, [ops::union], [UnionTy::from], and [UnionTy::add_constituents], maintain
/// this invariant.
///
/// TODO: Should we impose an order on constituent types?
///
/// ## Implementation detail
///
/// Union types are effectively sets of types, but since they usually are small (expected 2-3
/// constituents), we use a vector representation since linear operations are still acceptably
/// performant and the memory footprint is minimal.
///
/// Additionally, since it is important that Ty (and in turn `UnionTy`) is cheaply cloneable, we use
/// an `EcoVec` (which is COW) instead of a `Vec` to store the constituents.
#[derive(Debug, Clone)]
pub struct UnionTy(pub(crate) EcoVec<Ty>);

impl UnionTy {
    pub fn from(&self, constituents: impl IntoIterator<Item = Ty>) -> Ty {
        constituents.into_iter().fold(Ty::Never, |acc, ty| union(&acc, &ty))
    }

    pub fn contains(&self, ty: &Ty) -> bool {
        self.0.contains(ty)
    }

    pub fn add_constituents<'a>(&self, constituents: impl IntoIterator<Item = &'a Ty>) -> UnionTy {
        let mut new_constituents = self.0.clone();
        new_constituents.extend(constituents.into_iter().filter(|t| !self.contains(t)).cloned());
        UnionTy(new_constituents)
    }

    pub fn iter(&self) -> slice::Iter<'_, Ty> {
        self.0.iter()
    }
}

impl PartialEq for UnionTy {
    fn eq(&self, other: &UnionTy) -> bool {
        self.0.len() == other.0.len() && self.iter().all(|t| other.contains(t))
    }
}
impl Eq for UnionTy {}

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
