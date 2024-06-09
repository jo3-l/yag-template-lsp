use super::AstElement;
use crate::SyntaxElement;

pub trait UntypedToTypedExt {
    fn is<T: AstElement>(&self) -> bool {
        self.try_to::<T>().is_some()
    }

    fn to<T: AstElement>(&self) -> T {
        self.try_to().unwrap_or_else(|| {
            panic!("failed to cast node as `{:?}`", stringify!(T));
        })
    }

    fn try_to<T: AstElement>(&self) -> Option<T>;
}

impl<T: Into<SyntaxElement> + Clone> UntypedToTypedExt for T {
    fn try_to<A: AstElement>(&self) -> Option<A> {
        A::cast(self.clone().into())
    }
}
