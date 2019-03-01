use std::rc::Rc;

/// Wrap `Rc<String>` so that we can implement `Borrow<str>` on it
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct RcString(Rc<String>);

impl std::borrow::Borrow<str> for RcString {
    fn borrow(&self) -> &str {
        &self.0[..]
    }
}

impl From<Rc<String>> for RcString {
    fn from(rcs: Rc<String>) -> Self {
        RcString(rcs)
    }
}

impl From<RcString> for Rc<String> {
    fn from(rcs: RcString) -> Self {
        rcs.0
    }
}
