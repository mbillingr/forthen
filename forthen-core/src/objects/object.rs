use super::callable::Callable;
use super::dynobj::DynamicObject;
use super::prelude::*;
use crate::dictionary::WordId;
use crate::errors::*;
use crate::stack_effect::StackEffect;
use crate::state::State;
use crate::vm::ByteCode;
use std::any::Any;
use std::rc::Rc;

/// Dynamically typed value
#[derive(Clone)]
pub enum Object {
    None,
    False,
    True,
    I32(i32),
    Word(WordId),
    ByteCode(Rc<ByteCode>),
    Function(Callable),
    List(Rc<Vec<Object>>),
    String(Rc<String>),

    Dynamic(DynamicObject),

    Extension(Rc<ObjectInterface>),
}

impl std::fmt::Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.repr_sys())
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

impl std::cmp::PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        use Object::*;
        match (self, &other) {
            (None, None) => true,
            (Function(a), Function(b)) => a == b,
            (ByteCode(a), ByteCode(b)) => a == b,
            (List(_a), List(_b)) => unimplemented!(),
            (String(a), String(b)) => a == b,
            (I32(a), I32(b)) => a == b,
            (Extension(_a), _) => unimplemented!(),
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

impl From<bool> for Object {
    fn from(b: bool) -> Object {
        match b {
            true => Object::True,
            false => Object::False,
        }
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

    pub fn try_into_rc_quotation(self) -> Result<Rc<ByteCode>> {
        match self {
            Object::ByteCode(vec) => Ok(vec),
            _ => Err(ErrorKind::TypeError(format!("{:?} is no quotation", self)).into()),
        }
    }

    pub fn try_as_quotation_mut(&mut self) -> Result<&mut ByteCode> {
        match self {
            Object::ByteCode(vec) => Rc::get_mut(vec).ok_or(ErrorKind::OwnershipError.into()),
            _ => Err(ErrorKind::TypeError(format!("{:?} is not a quotation", self)).into()),
        }
    }
}

impl ObjectInterface for Object {
    fn as_any(&self) -> &dyn Any {
        match self {
            Object::Extension(dynobj) => dynobj.as_any(),
            _ => self,
        }
    }

    fn repr_sys(&self) -> String {
        match self {
            Object::None => format!("None"),
            Object::False => format!("False"),
            Object::True => format!("True"),
            Object::Word(id) => format!("{}", id),
            Object::ByteCode(q) => format!("[ {} ]", q),
            Object::Function(func) => format!("<{:?}>", func),
            Object::List(list) => format!("{:?}", list),
            Object::String(rcs) => format!("{:?}", rcs),
            Object::I32(i) => format!("{:?}", i),
            Object::Dynamic(dynobj) => dynobj.repr_sys(),
            Object::Extension(dynobj) => dynobj.repr_sys(),
        }
    }

    fn repr(&self, state: &mut State) -> Result<()> {
        match self {
            Object::Dynamic(dynobj) => dynobj.repr(state),
            Object::Extension(dynobj) => dynobj.repr(state),
            _ => state.push_string(self.repr_sys()),
        }
    }

    fn cmp_equal(&self, state: &mut State) -> Result<()> {
        use Object::*;
        let other = state.pop()?;
        let result = match (self, &other) {
            (None, None) => true,
            (Function(a), Function(b)) => a == b,
            (ByteCode(a), ByteCode(b)) => a == b,
            (List(_a), List(_b)) => unimplemented!(),
            (String(a), String(b)) => a == b,
            (I32(a), I32(b)) => a == b,
            (Dynamic(a), _) => {
                state.push(other)?;
                return a.cmp_equal(state);
            }
            (Extension(a), _) => {
                state.push(other)?;
                return a.cmp_equal(state);
            }
            _ => false,
        };
        state.push(result)?;
        Ok(())
    }

    fn is_number(&self) -> bool {
        match self {
            Object::I32(_) => true,
            Object::Dynamic(dynobj) => dynobj.is_number(),
            _ => false,
        }
    }

    fn is_callable(&self) -> bool {
        match self {
            Object::Word(_) | Object::Function(_) => true,
            Object::Dynamic(dynobj) => dynobj.is_callable(),
            _ => false,
        }
    }

    fn is_sequence(&self) -> bool {
        match self {
            Object::List(_) => true,
            Object::Dynamic(dynobj) => dynobj.is_sequence(),
            _ => false,
        }
    }

    fn get_stack_effect(&self) -> Result<&StackEffect> {
        match self {
            Object::Word(id) => id.word.inner().get_stack_effect(),
            Object::Function(f) => Ok(f.get_stack_effect()),
            Object::Dynamic(dynobj) => dynobj.get_stack_effect(),
            _ => panic!("{:?} is not callable", self),
        }
    }

    fn call(&self, state: &mut State) -> Result<()> {
        match self {
            Object::Word(id) => id.word.inner().call(state),
            Object::Function(f) => f.call(state),
            Object::Dynamic(dynobj) => dynobj.call(state),
            _ => Err(ErrorKind::TypeError(format!("{:?} is not callable", self)).into()),
        }
    }

    fn is_pure(&self) -> bool {
        match self {
            Object::Word(id) => id.word.inner().is_pure(),
            Object::Function(f) => f.is_pure(),
            Object::Dynamic(dynobj) => dynobj.is_pure(),
            _ => panic!("{:?} is not callable", self),
        }
    }

    fn as_vec_mut(&mut self) -> Result<&mut Vec<Object>> {
        match self {
            Object::List(vec) => Rc::get_mut(vec).ok_or(ErrorKind::OwnershipError.into()),
            Object::Dynamic(dynobj) => dynobj.as_vec_mut(),
            _ => Err(ErrorKind::TypeError(format!("{:?} is not a list", self)).into()),
        }
    }

    fn as_slice(&self) -> Result<&[Object]> {
        match self {
            Object::List(vec) => Ok(&vec),
            Object::Dynamic(dynobj) => dynobj.as_slice(),
            _ => Err(ErrorKind::TypeError(format!("{:?} is not a list", self)).into()),
        }
    }

    fn set_attr(&mut self, attr: Rc<String>, value: Object) {
        match self {
            Object::Dynamic(dynobj) => dynobj.set_attr(attr, value),
            _ => panic!("set attribute not implemented for {:?}", self),
        }
    }

    fn get_attr(&self, attr: &str) -> Option<Object> {
        match self {
            Object::Dynamic(dynobj) => dynobj.get_attr(attr),
            _ => None,
        }
    }

    fn set_attribute(&mut self, state: &mut State) -> Result<()> {
        match self {
            Object::Dynamic(dynobj) => dynobj.set_attribute(state),
            _ => Err(ErrorKind::TypeError(format!("get/set attribute not implemented for {:?}", self)).into()),
        }
    }
    fn get_attribute(&mut self, state: &mut State) -> Result<()> {
        match self {
            Object::Dynamic(dynobj) => dynobj.get_attribute(state),
            _ => Err(ErrorKind::TypeError(format!("get/set attribute not implemented for {:?}", self)).into()),
        }
    }

    fn add(&self, state: &mut State) -> Result<()> {
        use Object::*;
        let other = state.pop()?;
        match (self, &other) {
            (I32(a), I32(b)) => return state.push(I32(a + b)),
            (Extension(a), _) => {
                state.push(other)?;
                return a.add(state);
            }
            (Dynamic(a), _) => {
                state.push(other)?;
                return a.add(state);
            }
            (_, _) => {}
        }
        Err(ErrorKind::TypeError(format!("Cannot add {:?} + {:?}", self, other)).into())
    }
}
