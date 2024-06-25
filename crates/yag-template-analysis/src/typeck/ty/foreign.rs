use std::fmt;

use rustc_hash::FxHashMap;
use slotmap::{new_key_type, SlotMap};

use super::Ty;

#[derive(Debug)]
pub struct TypeDefinitions {
    pub initial_context_ty: Ty,
    pub funcs: FxHashMap<String, Func>,
    pub struct_types: SlotMap<StructHandle, StructTy>,
    pub callable_types: SlotMap<CallableHandle, CallableTy>,
    pub newtypes: SlotMap<NewtypeHandle, NewtypeTy>,
    pub map_types: SlotMap<MapHandle, MapTy>,
    pub typed_str_map_types: SlotMap<TypedStrMapHandle, TypedStrMapTy>,
    pub slice_types: SlotMap<SliceHandle, SliceTy>,
}

#[derive(Debug)]
pub struct Func {
    pub name: String,
    pub doc: String,
    pub call_signatures: Vec<CallSignature>,
}

impl fmt::Display for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "func {}", self.name)
    }
}

#[derive(Debug)]
pub enum CallSignature {
    Exact(Vec<Ty>),
    Variadic(Vec<Ty>, Ty),
    VariadicOptions(Vec<Ty>, FxHashMap<String, FuncOption>),
}

#[derive(Debug)]
pub struct FuncOption {
    pub required: bool,
    pub ty: Ty,
}

new_key_type! { pub struct StructHandle; }

#[derive(Debug)]
pub struct StructTy {
    pub name: String,
    pub doc: String,
    pub fields_and_methods: FxHashMap<String, FieldOrMethod>,
}

impl fmt::Display for StructTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

#[derive(Debug)]
pub enum FieldOrMethod {
    Field(Field),
    Method(CallableTy),
}

#[derive(Debug)]
pub struct Field {
    pub doc: String,
    pub ty: Ty,
}

new_key_type! { pub struct CallableHandle; }

#[derive(Debug)]
pub struct CallableTy {
    pub doc: String,
    pub call_signatures: Vec<CallSignature>,
}

impl fmt::Display for CallableTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("callable")
    }
}

new_key_type! { pub struct NewtypeHandle; }

#[derive(Debug)]
pub struct NewtypeTy {
    pub name: String,
    pub underlying: Ty,
    pub methods: FxHashMap<String, CallableTy>,
}

impl fmt::Display for NewtypeTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

new_key_type! { pub struct MapHandle; }

#[derive(Debug)]
pub struct MapTy {
    pub key_ty: Ty,
    pub value_ty: Ty,
}

new_key_type! { pub struct TypedStrMapHandle; }

#[derive(Debug)]
pub struct TypedStrMapTy {
    pub name: String,
    pub doc: String,
    pub fields: FxHashMap<String, Field>,
}

impl fmt::Display for TypedStrMapTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

new_key_type! { pub struct SliceHandle; }

#[derive(Debug)]
pub struct SliceTy {
    pub el_ty: Ty,
}
