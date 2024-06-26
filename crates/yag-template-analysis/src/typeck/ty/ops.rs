use ecow::eco_vec;

use super::{foreign, MapTy, PrimitiveTy, Ty, TypedStrMapTy, UnionTy};

pub fn indirect(mut ty: &Ty) -> &Ty {
    while let Ty::Pointer(derefs_to_ty) = ty {
        ty = &derefs_to_ty;
    }
    ty
}

pub fn underlying<'t, 'e>(ty: &'t Ty, defs: &'e foreign::TypeDefinitions) -> &'t Ty
where
    'e: 't,
{
    match indirect(ty) {
        Ty::Newtype(handle) => {
            let newtype = &defs.newtypes[*handle];
            underlying(&newtype.underlying, defs)
        }
        ty => ty,
    }
}

pub fn union(x: &Ty, y: &Ty) -> Ty {
    match (x, y) {
        (x, y) if x == y => x.clone(),
        (x, Ty::Never) => x.clone(),
        (Ty::Never, y) => y.clone(),

        (Ty::Any, _) | (_, Ty::Any) => Ty::Any,

        (Ty::Union(xs), Ty::Union(ys)) => Ty::Union(xs.add_constituents(ys.iter())),
        (Ty::Union(xs), y) => Ty::Union(xs.add_constituents([y])),
        (x, Ty::Union(ys)) => Ty::Union(ys.add_constituents([x])),

        (x, y) => Ty::Union(UnionTy(eco_vec![x.clone(), y.clone()])),
    }
}

/// Whether ty is loosely assignable to target_ty. The relation is symmetric, so
/// `loosely_assignable(x, y, env) <=> loosely_assignable(y, x, env)`.
pub fn loosely_assignable(ty: &Ty, target_ty: &Ty, defs: &foreign::TypeDefinitions) -> bool {
    let ty = underlying(ty, defs);
    let target_ty = underlying(target_ty, defs);
    match (ty, target_ty) {
        // `any` is loosely assignable to any other type (and vice versa.)
        (Ty::Any, _) => true,
        (_, Ty::Any) => true,

        // A union type U is loosely assignable to type T (and vice versa) if any constituent of U
        // is loosely assignable to T.
        (Ty::Union(xs), y) => xs.iter().any(|x_c| loosely_assignable(x_c, y, defs)),
        (x, Ty::Union(ys)) => ys.iter().any(|y_c| loosely_assignable(x, y_c, defs)),

        // Structs and callables are exact types.
        (Ty::Struct(handle), Ty::Struct(target_handle)) => handle == target_handle,
        (Ty::Method(handle), Ty::Method(target_handle)) => handle == target_handle,

        // Map types are loosely assignable to one another if their key types and value types are loosely assignable as well.
        (Ty::Map(xh), Ty::Map(yh)) => {
            let x = &defs.map_types[*xh];
            let y = &defs.map_types[*yh];
            loosely_assignable(&x.key_ty, &y.key_ty, defs) && loosely_assignable(&x.value_ty, &y.value_ty, defs)
        }
        // See comment on `map_typedstrmap_loosely_assignable`.
        (Ty::Map(xh), Ty::TypedStrMap(yh)) => {
            map_typedstrmap_loosely_assignable(&defs.map_types[*xh], &defs.typed_str_map_types[*yh], defs)
        }
        (Ty::TypedStrMap(xh), Ty::Map(yh)) => {
            map_typedstrmap_loosely_assignable(&defs.map_types[*yh], &defs.typed_str_map_types[*xh], defs)
        }

        // Slice types are loosely assignable if the element types are loosely assignable as well.
        (Ty::Slice(xh), Ty::Slice(yh)) => {
            let x = &defs.slice_types[*xh];
            let y = &defs.slice_types[*yh];
            loosely_assignable(&x.el_ty, &y.el_ty, defs)
        }

        // Primitive types are loosely assignable to one another if they are in the same class.
        (Ty::Primitive(x), Ty::Primitive(y)) => x.class() == y.class(),

        // Strings are loosely assignable to template names, and vice versa.
        (Ty::Primitive(PrimitiveTy::String), Ty::TemplateName) => true,
        (Ty::TemplateName, Ty::Primitive(PrimitiveTy::String)) => true,

        _ => false,
    }
}

/// A map type X is loosely assignable to a typed string map type Y (and vice versa) if the
/// following hold:
///
/// 1. X's key type is loosely assignable to type string.
/// 2. Each of Y's field types is loosely assignable to X's value type.
fn map_typedstrmap_loosely_assignable(x: &MapTy, y: &TypedStrMapTy, defs: &foreign::TypeDefinitions) -> bool {
    if !loosely_assignable(&x.key_ty, &Ty::Primitive(PrimitiveTy::String), defs) {
        return false;
    }

    // If X's value type is any, condition (2) trivially holds so we can forgo the loop.
    x.value_ty.is_any()
        || y.fields
            .values()
            .all(|y_field| loosely_assignable(&x.value_ty, &y_field.ty, defs))
}
