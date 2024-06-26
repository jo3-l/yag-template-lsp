use std::fmt;

use rustc_hash::FxHashMap;
use slotmap::{new_key_type, SlotMap};

use super::Ty;

#[derive(Debug)]
pub struct TypeDefinitions {
    pub initial_context_ty: Ty,
    pub funcs: FxHashMap<String, Func>,
    pub struct_types: SlotMap<StructHandle, StructTy>,
    pub method_types: SlotMap<MethodHandle, MethodTy>,
    pub newtypes: SlotMap<NewtypeHandle, NewtypeTy>,
    pub map_types: SlotMap<MapHandle, MapTy>,
    pub static_str_map_types: SlotMap<StaticStrMapHandle, StaticStrMapTy>,
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
pub struct CallSignature {
    pub kind: CallSignatureKind,
    pub ret_ty: Ty,
    pub throw_ty: Ty,
}

#[derive(Debug)]
pub enum CallSignatureKind {
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
    Method(MethodTy),
}

#[derive(Debug)]
pub struct Field {
    pub parent_ty_name: String,
    pub name: String,
    pub doc: String,
    pub ty: Ty,
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "field {}.{}", self.parent_ty_name, self.name)
    }
}

new_key_type! { pub struct MethodHandle; }

#[derive(Debug)]
pub struct MethodTy {
    pub recv_ty_name: String,
    pub name: String,
    pub doc: String,
    pub call_signatures: Vec<CallSignature>,
}

impl fmt::Display for MethodTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "func {}.{}", self.recv_ty_name, self.name)
    }
}

new_key_type! { pub struct NewtypeHandle; }

#[derive(Debug)]
pub struct NewtypeTy {
    pub name: String,
    pub underlying: Ty,
    pub methods: FxHashMap<String, MethodTy>,
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

new_key_type! { pub struct StaticStrMapHandle; }

#[derive(Debug)]
pub struct StaticStrMapTy {
    pub name: String,
    pub doc: String,
    pub fields: FxHashMap<String, Field>,
}

impl fmt::Display for StaticStrMapTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

new_key_type! { pub struct SliceHandle; }

#[derive(Debug)]
pub struct SliceTy {
    pub el_ty: Ty,
}
