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

    /// convert into reference counted `String`
    pub fn into_rc_string(self) -> Rc<String> {
        match self {
            Object::String(rcs) => rcs,
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
