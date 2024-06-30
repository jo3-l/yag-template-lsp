use std::mem::swap;
use std::slice;

use ecow::{eco_vec, EcoVec};
use itertools::Itertools;

use super::Ty;

pub fn union_all<'a>(types: impl IntoIterator<Item = &'a Ty>) -> Ty {
    types.into_iter().fold(Ty::Never, |acc, ty| union(&acc, ty))
}

pub fn union(x: &Ty, y: &Ty) -> Ty {
    match (x, y) {
        (x, y) if x == y => x.clone(),
        (x, Ty::Never) => x.clone(),
        (Ty::Never, y) => y.clone(),

        (Ty::Any, _) | (_, Ty::Any) => Ty::Any,

        (Ty::Union(xs), Ty::Union(ys)) => Ty::Union(xs.add_all(ys)),
        (Ty::Union(xs), y) => Ty::Union(xs.add(y)),
        (x, Ty::Union(ys)) => Ty::Union(ys.add(x)),

        (x, y) => {
            let mut x = x.clone();
            let mut y = y.clone();
            if x > y {
                swap(&mut x, &mut y);
            };
            Ty::Union(UnionTy(eco_vec![x, y]))
        }
    }
}

/// A union of multiple types in sorted order.
///
/// By construction, union types always have two or more constituents. All operations that produce
/// or modify union types maintain this invariant.
///
/// ## Implementation detail
///
/// Union types are internally modelled as an immutable, sorted vector. Although modifying existing
/// union type requires cloning under this representation, data structures that would be more
/// efficient in theory — namely, persistent data structures — are not worth considering since most
/// union types are small. (For similar reasons, we do not represent union types as a set.)
///
/// Additionally, since it is important that types are cheaply cloneable, we use an `EcoVec` instead
/// of a plain `Vec` so that clones reuse the same memory location.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct UnionTy(EcoVec<Ty>);

impl UnionTy {
    pub fn iter(&self) -> slice::Iter<'_, Ty> {
        self.0.iter()
    }

    fn add(&self, ty: &Ty) -> UnionTy {
        match self.0.binary_search(ty) {
            Ok(_) => self.clone(),
            Err(pos) => {
                let mut new_constituents = self.0.clone();
                new_constituents.insert(pos, ty.clone());
                UnionTy(new_constituents)
            }
        }
    }

    fn add_all(&self, other: &UnionTy) -> UnionTy {
        let constituents = self.iter().merge(other.iter()).dedup();
        UnionTy(constituents.cloned().collect())
    }
}
