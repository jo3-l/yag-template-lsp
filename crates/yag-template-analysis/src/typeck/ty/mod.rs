use std::fmt;

mod display;
pub mod foreign;
mod relation;
mod union;

pub use display::TyDisplay;
pub use foreign::{
    MapHandle, MapTy, MethodHandle, MethodTy, NewtypeHandle, NewtypeTy, SliceHandle, SliceTy, StaticStrMapHandle,
    StaticStrMapTy, StructHandle, StructTy,
};
pub use relation::loosely_assignable;
pub use union::{union, UnionTy};

pub fn indirect(mut ty: &Ty) -> &Ty {
    while let Ty::Pointer(derefs_to_ty) = ty {
        ty = derefs_to_ty;
    }
    ty
}

pub fn base_ty<'t, 'e>(ty: &'t Ty, defs: &'e foreign::TypeDefinitions) -> &'t Ty
where
    'e: 't,
{
    match indirect(ty) {
        Ty::Newtype(handle) => {
            let newtype = &defs.newtypes[*handle];
            base_ty(&newtype.underlying, defs)
        }
        ty => ty,
    }
}

/// An immutable, cheaply cloneable representation of a type in YAGPDB's templating system.
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
