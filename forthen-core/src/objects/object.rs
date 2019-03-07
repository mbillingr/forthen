use crate::dictionary::WordId;
use crate::errors::*;
use std::rc::Rc;
//use crate::scope::ScopeDef;
use super::callable::{Callable};
use super::prelude::*;
use crate::stack_effect::StackEffect;
use crate::state::State;
use crate::vm::Quotation;

/// Dynamically typed value
#[derive(Clone)]
pub enum Object {
    None,
    False,
    True,
    Word(WordId),
    Quotation(Rc<Quotation>, StackEffect),
    /*NativeFunction(NativeFunction, StackEffect),
    NativeClosure(NativeClosure, StackEffect),*/
    Function(Callable),
    //CompoundFunction(Rc<Vec<Object>>, StackEffect),
    //ScopedFunction(Rc<Vec<Object>>, StackEffect, ScopeDef),
    List(Rc<Vec<Object>>),
    String(Rc<String>),
    I32(i32),

    Dynamic(Rc<DynamicObject>),
}

impl std::fmt::Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Object::None => write!(f, "None"),
            Object::False => write!(f, "False"),
            Object::True => write!(f, "True"),
            Object::Word(id) => write!(f, "{}", id),
            Object::Quotation(q, _) => write!(f, "[ {} ]", q),
            Object::Function(func) => write!(f, "<{:?}>", func),
            /*Object::NativeFunction(_, se) => write!(f, "<native ({})>", se),
            Object::NativeClosure(_, se) => write!(f, "<closure ({})>", se),*/
            Object::List(list) => write!(f, "{:?}", list),
            Object::String(rcs) => write!(f, "{:?}", rcs),
            Object::I32(i) => write!(f, "{:?}", i),
            Object::Dynamic(dynobj) => write!(f, "{}", dynobj.repr()),
        }
    }
}

impl std::cmp::PartialEq for Object {
    fn eq(&self, other: &Object) -> bool {
        use Object::*;
        match (self, other) {
            (None, None) => true,
            (Function(a), Function(b)) => a == b,
            /*(NativeFunction(a, _), NativeFunction(b, _)) => a as *const _ == b as *const _,
            (NativeClosure(a, _), NativeClosure(b, _)) => &**a as *const _ == &**b as *const _,*/
            (Quotation(a, _), Quotation(b, _)) => a == b,
            (List(a), List(b)) => a == b,
            (String(a), String(b)) => a == b,
            (I32(a), I32(b)) => a == b,
            (Dynamic(a), Dynamic(b)) => DynamicObject::eq(&**a, &**b),
            _ => false,
        }
    }
}

impl std::ops::Add for Object {
    type Output = Result<Object>;
    fn add(self, other: Object) -> Self::Output {
        use Object::*;
        match (self, other) {
            (I32(a), I32(b)) => Ok(I32(a + b)),
            (a, b) => Err(ErrorKind::TypeError(format!("Cannot add {:?} + {:?}", a, b)).into()),
        }
    }
}

impl std::ops::Sub for Object {
    type Output = Result<Object>;
    fn sub(self, other: Object) -> Self::Output {
        use Object::*;
        match (self, other) {
            (I32(a), I32(b)) => Ok(I32(a - b)),
            (a, b) => {
                Err(ErrorKind::TypeError(format!("Cannot subtract {:?} - {:?}", a, b)).into())
            }
        }
    }
}

impl std::ops::Mul for Object {
    type Output = Result<Object>;
    fn mul(self, other: Object) -> Self::Output {
        use Object::*;
        match (self, other) {
            (I32(a), I32(b)) => Ok(I32(a * b)),
            (a, b) => {
                Err(ErrorKind::TypeError(format!("Cannot multiply {:?} * {:?}", a, b)).into())
            }
        }
    }
}

impl std::ops::Div for Object {
    type Output = Result<Object>;
    fn div(self, other: Object) -> Self::Output {
        use Object::*;
        match (self, other) {
            (I32(a), I32(b)) => Ok(I32(a / b)),
            (a, b) => Err(ErrorKind::TypeError(format!("Cannot divide {:?} / {:?}", a, b)).into()),
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
        obj.try_into_rc_string().unwrap()
    }
}

impl From<Object> for i32 {
    fn from(obj: Object) -> Self {
        obj.try_into_i32().unwrap()
    }
}

impl Object {
    pub fn get_stack_effect(&self) -> Result<StackEffect> {
        match self {
            Object::Word(id) => id.word.inner().get_stack_effect(),
            Object::Quotation(_, se)
            /*| Object::NativeFunction(_, se)
            | Object::NativeClosure(_, se)*/ => Ok(se.clone()),
            Object::Function(f) => Ok(f.get_stack_effect().clone()),
            _ => Err(ErrorKind::TypeError(format!("{:?} is not callable", self)).into()),
        }
    }

    /// if the object is callable, call it otherwise push itself on stack.
    pub fn invoke(&self, state: &mut State) -> Result<()> {
        match self {
            Object::Word(id) => id.word.inner().invoke(state),
            Object::Quotation(quot, _) => quot.run(state),
            /*Object::NativeFunction(fun, _) => fun(state),
            Object::NativeClosure(fun, _) => fun(state),*/
            Object::Function(f) => f.call(state),
            _ => Err(ErrorKind::TypeError(format!("{:?} is not callable", self)).into()),
        }
    }

    /// allows to mutate a List object if there is no other reference to it
    pub fn as_vec_mut(&mut self) -> Result<&mut Vec<Object>> {
        match self {
            Object::List(vec) => Rc::get_mut(vec).ok_or(ErrorKind::OwnershipError.into()),
            _ => Err(ErrorKind::TypeError(format!("{:?} is not a list", self)).into()),
        }
    }

    /// view the object as a slice
    ///
    /// Supported variants: `List`
    pub fn as_slice(&self) -> Result<&[Object]> {
        match self {
            Object::List(vec) => Ok(&vec),
            _ => Err(ErrorKind::TypeError(format!("{:?} is not a list", self)).into()),
        }
    }

    /// convert into reference counted `Vec`
    pub fn into_rc_vec(self) -> Result<Rc<Vec<Object>>> {
        match self {
            Object::List(vec) => Ok(vec),
            _ => Err(ErrorKind::TypeError(format!("{:?} is not a list", self)).into()),
        }
    }

    /// try to convert into `i32`.
    pub fn try_into_bool(self) -> Result<bool> {
        match self {
            Object::True => Ok(true),
            Object::False => Ok(false),
            _ => Err(ErrorKind::TypeError(format!("{:?} is not boolean", self)).into()),
        }
    }

    /// try to convert into `i32`.
    pub fn try_into_i32(self) -> Result<i32> {
        match self {
            Object::I32(i) => Ok(i),
            _ => Err(ErrorKind::TypeError(format!("{:?} is no integer", self)).into()),
        }
    }

    /// try to convert into `i32`.
    pub fn try_into_rc_string(self) -> Result<Rc<String>> {
        match self {
            Object::String(rcs) => Ok(rcs),
            _ => Err(ErrorKind::TypeError(format!("{:?} is no string", self)).into()),
        }
    }

    pub fn try_into_rc_quotation(self) -> Result<Rc<Quotation>> {
        match self {
            Object::Quotation(vec, _) => Ok(vec),
            _ => Err(ErrorKind::TypeError(format!("{:?} is no quotation", self)).into()),
        }
    }

    pub fn try_as_quotation_mut(&mut self) -> Result<&mut Quotation> {
        match self {
            Object::Quotation(vec, _) => Rc::get_mut(vec).ok_or(ErrorKind::OwnershipError.into()),
            _ => Err(ErrorKind::TypeError(format!("{:?} is not a quotation", self)).into()),
        }
    }
}
