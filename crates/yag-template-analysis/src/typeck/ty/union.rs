use std::slice;
use std::sync::Arc;

use ecow::{eco_vec, EcoVec};
use itertools::Itertools;

use super::Ty;

/// A union of multiple types in sorted order.
///
/// By construction, union types always have two or more constituents. All operations that produce
/// or modify union types maintain this invariant.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct UnionTy(Repr);

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
            let sorted_xy = algo::sort2(x.clone(), y.clone()).into();
            Ty::Union(UnionTy(Repr::Small(Arc::new(sorted_xy))))
        }
    }
}

impl UnionTy {
    pub fn iter(&self) -> slice::Iter<'_, Ty> {
        match &self.0 {
            Repr::Small(constituents) => constituents.iter(),
            Repr::Large(constituents) => constituents.iter(),
        }
    }

    fn add(&self, ty: &Ty) -> UnionTy {
        match &self.0 {
            Repr::Small(constituents) => {
                if constituents.contains(ty) {
                    //
                    self.clone()
                } else {
                    let (a, b, c) = algo::sort3(constituents[0].clone(), constituents[1].clone(), ty.clone());
                    UnionTy(Repr::Large(eco_vec![a, b, c]))
                }
            }
            Repr::Large(constituents) => match constituents.binary_search(ty) {
                Ok(_) => self.clone(),
                Err(pos) => {
                    let mut new_constituents = constituents.clone();
                    new_constituents.insert(pos, ty.clone());
                    UnionTy(Repr::Large(new_constituents))
                }
            },
        }
    }

    fn add_all(&self, other: &UnionTy) -> UnionTy {
        let constituents = self.iter().merge(other.iter()).dedup();
        UnionTy(Repr::Large(constituents.cloned().collect()))
    }
}

/// The internal representation of a union type: an immutable, sorted vector. Although modifying
/// existing union type requires cloning under this representation, data structures that would be
/// more efficient in theory — namely, persistent data structures — are not worth considering since
/// most union types are small. (For similar reasons, we do not represent union types as a set.)
///
/// Additionally, since it is important that types are cheaply cloneable, instead of using a plain
/// `Vec`, we use either an `Arc<[Ty; 2]>` (for the overwhelming common case of two constituents) or
/// an `EcoVec`, so that clones reuse the same memory location.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Repr {
    // Unions with two constituents are the common case, so optimize for them.
    Small(Arc<[Ty; 2]>),
    Large(EcoVec<Ty>),
}

mod algo {
    use std::mem::swap;

    pub(super) fn sort2<T: PartialOrd>(a: T, b: T) -> (T, T) {
        if a <= b {
            (a, b)
        } else {
            (b, a)
        }
    }

    pub(super) fn sort3<T: PartialOrd>(mut a: T, mut b: T, mut c: T) -> (T, T, T) {
        if a > b {
            swap(&mut a, &mut b);
        }
        if b > c {
            swap(&mut b, &mut c);
        }
        if a > b {
            swap(&mut a, &mut b);
        }
        (a, b, c)
    }
}
