use super::{base_ty, foreign, MapTy, PrimitiveTy, StaticStrMapTy, Ty};

/// Whether type x is loosely assignable to type y. The relation is symmetric, so
/// `loosely_assignable(x, y) <=> loosely_assignable(y, x)`.
pub fn loosely_assignable(x: &Ty, y: &Ty, defs: &foreign::TypeDefinitions) -> bool {
    let x = base_ty(x, defs);
    let y = base_ty(y, defs);
    match (x, y) {
        // `any` is loosely assignable to any other type (and vice versa.)
        (Ty::Any, _) | (_, Ty::Any) => true,

        // A union type U is loosely assignable to type T (and vice versa) if any constituent of U
        // is loosely assignable to T.
        (Ty::Union(xs), y) => xs.iter().any(|x_c| loosely_assignable(x_c, y, defs)),
        (x, Ty::Union(ys)) => ys.iter().any(|y_c| loosely_assignable(x, y_c, defs)),

        // Structs and callables are loosely assignable to one another if they have the same
        // identity.
        (Ty::Struct(xh), Ty::Struct(yh)) => xh == yh,
        (Ty::Method(xh), Ty::Method(yh)) => xh == yh,

        // Map types are loosely assignable to one another if their key types and value types are
        // loosely assignable to one another.
        (Ty::Map(xh), Ty::Map(yh)) => {
            let x = &defs.map_types[*xh];
            let y = &defs.map_types[*yh];
            loosely_assignable(&x.key_ty, &y.key_ty, defs) && loosely_assignable(&x.value_ty, &y.value_ty, defs)
        }
        // Static string maps are loosely assignable to one another if they have the same identity.
        (Ty::StaticStrMap(xh), Ty::StaticStrMap(yh)) => xh == yh,
        // See comment on `map_staticstrmap_loosely_assignable`.
        (Ty::Map(xh), Ty::StaticStrMap(yh)) => {
            map_staticstrmap_loosely_assignable(&defs.map_types[*xh], &defs.static_str_map_types[*yh], defs)
        }
        (Ty::StaticStrMap(xh), Ty::Map(yh)) => {
            map_staticstrmap_loosely_assignable(&defs.map_types[*yh], &defs.static_str_map_types[*xh], defs)
        }

        // Slice types are loosely assignable to one another if their element types are loosely
        // assignable to one another.
        (Ty::Slice(xh), Ty::Slice(yh)) => {
            let x = &defs.slice_types[*xh];
            let y = &defs.slice_types[*yh];
            loosely_assignable(&x.el_ty, &y.el_ty, defs)
        }

        // Primitive types are loosely assignable to one another if they share the same class.
        (Ty::Primitive(x), Ty::Primitive(y)) => x.class() == y.class(),

        // Strings are assignable to template names, and vice versa.
        (Ty::Primitive(PrimitiveTy::String), Ty::TemplateName)
        | (Ty::TemplateName, Ty::Primitive(PrimitiveTy::String)) => true,

        _ => false,
    }
}

/// A map type X is loosely assignable to a static string map type Y (and vice versa) if the
/// following hold:
///
/// 1. X's key type is loosely assignable to type string.
/// 2. Each of Y's field types is loosely assignable to X's value type.
fn map_staticstrmap_loosely_assignable(x: &MapTy, y: &StaticStrMapTy, defs: &foreign::TypeDefinitions) -> bool {
    if !loosely_assignable(&x.key_ty, &Ty::Primitive(PrimitiveTy::String), defs) {
        return false;
    }

    // If X's value type is any, condition (2) trivially holds so we can skip the loop.
    x.value_ty.is_any()
        || y.fields
            .values()
            .all(|y_field| loosely_assignable(&x.value_ty, &y_field.ty, defs))
}
