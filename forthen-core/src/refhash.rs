use std::rc::Rc;
use std::hash::{Hash, Hasher};

/// Helper for hashing by Rc identity
pub struct RefHash<T> {
    inner: Rc<T>,
}

impl<T> RefHash<T> {
    pub fn new(inner: Rc<T>) -> Self {
        RefHash { inner }
    }
}

impl<T: std::fmt::Debug> RefHash<T> {
    pub fn into_deref(self) -> T {
        Rc::try_unwrap(self.inner).unwrap()
    }
}

impl<T> Hash for RefHash<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let ptr = Rc::into_raw(self.inner.clone());
        ptr.hash(state);
        let _ = unsafe { Rc::from_raw(ptr) };
    }
}

impl<T> PartialEq for RefHash<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}
impl<T> Eq for RefHash<T> {}

impl<T> Clone for RefHash<T> {
    fn clone(&self) -> Self {
        RefHash {
            inner: self.inner.clone(),
        }
    }
}

impl<T> std::ops::Deref for RefHash<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &*self.inner
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for RefHash<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}
