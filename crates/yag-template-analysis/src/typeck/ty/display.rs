use std::fmt::{self, Write};

use super::foreign::TypeDefinitions;
use super::Ty;

pub struct TyDisplay<'e> {
    ty: Ty,
    env: &'e TypeDefinitions,
}

impl Ty {
    pub fn display<'e>(&self, env: &'e TypeDefinitions) -> TyDisplay<'e> {
        TyDisplay { ty: self.clone(), env }
    }
}

impl fmt::Display for TyDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Ty::*;
        match &self.ty {
            Any => f.write_str("any")?,
            Union(constituents) => {
                let mut constituents = constituents.iter();
                if let Some(first_ty) = constituents.next() {
                    f.write_char('(')?;
                    first_ty.display(self.env).fmt(f)?;
                    for ty in constituents {
                        f.write_str(" | ")?;
                        ty.display(self.env).fmt(f)?;
                    }
                    f.write_char(')')?;
                } else {
                    f.write_str("never")?;
                }
            }
            Pointer(derefs_to_ty) => write!(f, "*{}", derefs_to_ty.display(self.env))?,

            Struct(h) => self.env.struct_types[*h].fmt(f)?,
            Callable(h) => self.env.callable_types[*h].fmt(f)?,
            Newtype(h) => self.env.newtypes[*h].fmt(f)?,
            Map(h) => {
                let map_ty = &self.env.map_types[*h];
                write!(
                    f,
                    "map[{key}]{value}",
                    key = map_ty.key_ty.display(self.env),
                    value = map_ty.value_ty.display(self.env)
                )?;
            }
            TypedStrMap(h) => self.env.typed_str_map_types[*h].fmt(f)?,
            Slice(h) => {
                let slice_ty = &self.env.slice_types[*h];
                write!(f, "[]{el}", el = slice_ty.el_ty.display(self.env))?;
            }

            Primitive(p) => p.fmt(f)?,
        }
        Ok(())
    }
}