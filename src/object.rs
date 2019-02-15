use std::rc::Rc;

use crate::state::State;

/// Dynamically typed value
#[derive(Clone)]
pub enum Object {
    None,
    NativeFunction(fn(&mut State)),
    CompoundFunction(Rc<Vec<Object>>),
    List(Rc<Vec<Object>>),
    String(Rc<String>),
    I32(i32),
}

impl std::fmt::Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Object::None => write!(f, "None"),
            Object::NativeFunction(_) => write!(f, "<native>"),
            Object::CompoundFunction(ops) => write!(f, "{:?}", ops),
            Object::List(list) => write!(f, "{:?}", list),
            Object::String(rcs) => write!(f, "{:?}", rcs),
            Object::I32(i) => write!(f, "{:?}", i),
        }
    }
}

impl std::cmp::PartialEq for Object {
    fn eq(&self, other: &Object) -> bool {
        use Object::*;
        match (self, other) {
            (None, None) => true,
            (NativeFunction(a), NativeFunction(b)) => a as *const _ == b as *const _,
            (CompoundFunction(a), CompoundFunction(b)) => a == b,
            (List(a), List(b)) => a == b,
            (String(a), String(b)) => a == b,
            (I32(a), I32(b)) => a == b,
            _ => false,
        }
    }
}

impl std::cmp::PartialEq<i32> for Object {
    fn eq(&self, other: &i32) -> bool {
        match self {
            Object::I32(i) => i == other,
            _ => false,
        }
    }
}

impl std::cmp::PartialEq<&str> for Object {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Object::String(s) => &**s == other,
            _ => false,
        }
    }
}

impl From<Rc<String>> for Object {
    fn from(s: Rc<String>) -> Object {
        Object::String(s)
    }
}

impl From<i32> for Object {
    fn from(i: i32) -> Object {
        Object::I32(i)
    }
}

impl From<Object> for Rc<String> {
    fn from(obj: Object) -> Self {
        match obj {
            Object::String(rcs) => rcs,
            _ => panic!("Type Error"),
        }
    }
}

impl From<Object> for i32 {
    fn from(obj: Object) -> Self {
        match obj {
            Object::I32(i) => i,
            _ => panic!("Type Error"),
        }
    }
}

impl Object {
    /// if the object is callable, call it otherwise push itself on stack.
    pub fn invoke(self, state: &mut State) {
        match self {
            Object::NativeFunction(fun) => fun(state),
            Object::CompoundFunction(ops) => state.run_sequence(&ops[..]),
            other => state.push(other),
        }
    }

    /// allows to mutate a List object if there is no other reference to it
    pub fn as_vec_mut(&mut self) -> &mut Vec<Object> {
        match self {
            Object::List(vec) => Rc::get_mut(vec).expect("Unable to mutate list"),
            _ => panic!("Type Error"),
        }
    }

    /// view the object as a slice
    ///
    /// Supported variants: `List`
    pub fn as_slice(&self) -> &[Object] {
        match self {
            Object::List(vec) => &vec,
            _ => panic!("Type Error"),
        }
    }

    /// convert into reference counted `Vec`
    pub fn into_rc_vec(self) -> Rc<Vec<Object>> {
        match self {
            Object::List(vec) => vec,
            _ => panic!("Type Error"),
        }
    }

    /// try to convert into `i32`.
    pub fn try_into_i32(self) -> Option<i32> {
        match self {
            Object::I32(i) => Some(i),
            _ => None,
        }
    }
}
