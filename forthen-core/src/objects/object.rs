use super::callable::Callable;
use super::prelude::*;
use super::table::Table;
use crate::dictionary::WordId;
use crate::errors::*;
use crate::objects::table::TableImpl;
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

    Table(Table),

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
    pub fn new_table() -> Self {
        Object::Table(Rc::new(TableImpl::new()))
    }

    pub fn is_same(&self, other: &Object) -> bool {
        use Object::*;
        match (self, other) {
            (None, None) => true,
            (False, False) => true,
            (True, True) => true,
            (I32(a), I32(b)) => a == b,
            (Word(a), Word(b)) => Rc::ptr_eq(a, b),
            (Function(a), Function(b)) => a == b,
            (ByteCode(a), ByteCode(b)) => Rc::ptr_eq(a, b),
            (List(a), List(b)) => Rc::ptr_eq(a, b),
            (String(a), String(b)) => Rc::ptr_eq(a, b),
            (Table(a), Table(b)) => Rc::ptr_eq(a, b),
            (Extension(a), Extension(b)) => Rc::ptr_eq(a, b),
            _ => false,
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
            Object::Table(dynobj) => dynobj.repr_sys(),
            Object::Extension(dynobj) => dynobj.repr_sys(),
        }
    }

    fn repr(&self, state: &mut State) -> Result<()> {
        match self {
            Object::Table(dynobj) => dynobj.repr(state),
            Object::Extension(dynobj) => dynobj.repr(state),
            _ => state.push_string(self.repr_sys()),
        }
    }

    fn is_number(&self) -> bool {
        match self {
            Object::I32(_) => true,
            Object::Table(dynobj) => dynobj.is_number(),
            _ => false,
        }
    }

    fn is_callable(&self) -> bool {
        match self {
            Object::Word(_) | Object::Function(_) => true,
            Object::Table(dynobj) => dynobj.is_callable(),
            _ => false,
        }
    }

    fn is_sequence(&self) -> bool {
        match self {
            Object::List(_) => true,
            Object::Table(dynobj) => dynobj.is_sequence(),
            _ => false,
        }
    }

    fn get_stack_effect(&self) -> Result<&StackEffect> {
        match self {
            Object::Word(id) => id.word.inner().get_stack_effect(),
            Object::Function(f) => Ok(f.get_stack_effect()),
            Object::Table(dynobj) => dynobj.get_stack_effect(),
            _ => panic!("{:?} is not callable", self),
        }
    }

    fn call(&self, state: &mut State) -> Result<()> {
        match self {
            Object::Word(id) => id.word.inner().call(state),
            Object::Function(f) => f.call(state),
            Object::Table(dynobj) => dynobj.call(state),
            _ => Err(ErrorKind::TypeError(format!("{:?} is not callable", self)).into()),
        }
    }

    fn is_pure(&self) -> bool {
        match self {
            Object::Word(id) => id.word.inner().is_pure(),
            Object::Function(f) => f.is_pure(),
            Object::Table(dynobj) => dynobj.is_pure(),
            _ => panic!("{:?} is not callable", self),
        }
    }

    fn as_vec_mut(&mut self) -> Result<&mut Vec<Object>> {
        match self {
            Object::List(vec) => Rc::get_mut(vec).ok_or(ErrorKind::OwnershipError.into()),
            Object::Table(dynobj) => dynobj.as_vec_mut(),
            _ => Err(ErrorKind::TypeError(format!("{:?} is not a list", self)).into()),
        }
    }

    fn as_slice(&self) -> Result<&[Object]> {
        match self {
            Object::List(vec) => Ok(&vec),
            Object::Table(dynobj) => dynobj.as_slice(),
            _ => Err(ErrorKind::TypeError(format!("{:?} is not a list", self)).into()),
        }
    }

    fn set_meta(&mut self, meta: Option<Table>) -> Result<()> {
        match self {
            Object::Table(table) => {
                let t: Result<_> = Rc::get_mut(table).ok_or(ErrorKind::OwnershipError.into());
                t?.set_metatable(meta);
                Ok(())
            }
            _ => Err(ErrorKind::TypeError(format!(
                "{:?} can't have a meta table",
                self.repr_sys()
            ))
            .into()),
        }
    }

    fn get_meta(&mut self) -> Option<Table> {
        match self {
            Object::Table(table) => table.get_metatable().cloned(),
            _ => None,
        }
    }

    fn set_attr(&mut self, attr: Rc<String>, value: Object) {
        match self {
            Object::Table(dynobj) => dynobj.set_attr(attr, value),
            _ => panic!("set attribute not implemented for {:?}", self),
        }
    }

    fn get_attr(&self, attr: &str) -> Option<Object> {
        match self {
            Object::Table(dynobj) => dynobj.get_attr(attr),
            _ => None,
        }
    }

    fn set_attribute(&mut self, state: &mut State) -> Result<()> {
        match self {
            Object::Table(dynobj) => dynobj.set_attribute(state),
            _ => Err(ErrorKind::TypeError(format!(
                "get/set attribute not implemented for {:?}",
                self
            ))
            .into()),
        }
    }
    fn get_attribute(&self, state: &mut State) -> Result<()> {
        match self {
            Object::Table(dynobj) => dynobj.get_attribute(state),
            _ => Err(ErrorKind::TypeError(format!(
                "get/set attribute not implemented for {:?}",
                self
            ))
            .into()),
        }
    }

    fn not(&self, state: &mut State) -> Result<()> {
        match self {
            Object::True => state.push(Object::False),
            Object::False => state.push(Object::True),
            _ => Err(ErrorKind::TypeError(format!("Cannot logically invert {:?}", self)).into()),
        }
    }

    fn is_eq(&self, state: &mut State) -> Result<()> {
        use Object::*;
        let other = state.pop()?;
        match (self, &other) {
            (None, None) => state.push(true),
            (False, False) => state.push(true),
            (True, True) => state.push(true),
            (I32(a), I32(b)) => state.push(a == b),
            (String(a), String(b)) => state.push(a == b),
            (Table(a), _) => {
                state.push(other)?;
                a.is_eq(state)
            }
            (Extension(a), _) => {
                state.push(other)?;
                a.is_eq(state)
            }
            _ => Err(ErrorKind::TypeError(format!(
                "is_eq not implemented for {:?} and {:?}",
                self.repr_sys(), other
            )).into()),
        }
    }

    fn is_gt(&self, state: &mut State) -> Result<()> {
        use Object::*;
        let other = state.pop()?;
        match (self, &other) {
            (I32(a), I32(b)) => state.push(a > b),
            (Table(a), _) => {
                state.push(other)?;
                a.is_gt(state)
            }
            (Extension(a), _) => {
                state.push(other)?;
                a.is_gt(state)
            }
            _ => Err(ErrorKind::TypeError(format!(
                "is_gt not implemented for {:?} and {:?}",
                self.repr_sys(), other
            )).into()),
        }
    }

    fn is_lt(&self, state: &mut State) -> Result<()> {
        use Object::*;
        let other = state.pop()?;
        match (self, &other) {
            (I32(a), I32(b)) => state.push(a < b),
            (Table(a), _) => {
                state.push(other)?;
                a.is_lt(state)
            }
            (Extension(a), _) => {
                state.push(other)?;
                a.is_lt(state)
            }
            _ => Err(ErrorKind::TypeError(format!(
                "is_lt not implemented for {:?} and {:?}",
                self.repr_sys(), other
            )).into()),
        }
    }

    fn is_ge(&self, state: &mut State) -> Result<()> {
        use Object::*;
        let other = state.pop()?;
        match (self, &other) {
            (I32(a), I32(b)) => state.push(a >= b),
            (Table(a), _) => {
                state.push(other)?;
                a.is_ge(state)
            }
            (Extension(a), _) => {
                state.push(other)?;
                a.is_ge(state)
            }
            _ => Err(ErrorKind::TypeError(format!(
                "is_ge not implemented for {:?} and {:?}",
                self.repr_sys(), other
            )).into()),
        }
    }

    fn is_le(&self, state: &mut State) -> Result<()> {
        use Object::*;
        let other = state.pop()?;
        match (self, &other) {
            (I32(a), I32(b)) => state.push(a <= b),
            (Table(a), _) => {
                state.push(other)?;
                a.is_le(state)
            }
            (Extension(a), _) => {
                state.push(other)?;
                a.is_le(state)
            }
            _ => Err(ErrorKind::TypeError(format!(
                "is_le not implemented for {:?} and {:?}",
                self.repr_sys(), other
            )).into()),
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
            (Table(a), _) => {
                state.push(other)?;
                return a.add(state);
            }
            (_, _) => {}
        }
        Err(ErrorKind::TypeError(format!("Cannot add {:?} + {:?}", self, other)).into())
    }

    fn sub(&self, state: &mut State) -> Result<()> {
        use Object::*;
        let other = state.pop()?;
        match (self, &other) {
            (I32(a), I32(b)) => return state.push(I32(b - a)),
            (Extension(a), _) => {
                state.push(other)?;
                return a.sub(state);
            }
            (Table(a), _) => {
                state.push(other)?;
                return a.sub(state);
            }
            (_, _) => {}
        }
        Err(ErrorKind::TypeError(format!("Cannot subtract {:?} - {:?}", other, self)).into())
    }

    fn mul(&self, state: &mut State) -> Result<()> {
        use Object::*;
        let other = state.pop()?;
        match (self, &other) {
            (I32(a), I32(b)) => return state.push(I32(a * b)),
            (Extension(a), _) => {
                state.push(other)?;
                return a.mul(state);
            }
            (Table(a), _) => {
                state.push(other)?;
                return a.mul(state);
            }
            (_, _) => {}
        }
        Err(ErrorKind::TypeError(format!("Cannot multiply {:?} * {:?}", self, other)).into())
    }

    fn div(&self, state: &mut State) -> Result<()> {
        use Object::*;
        let other = state.pop()?;
        match (self, &other) {
            (I32(a), I32(b)) => return state.push(I32(b / a)),
            (Extension(a), _) => {
                state.push(other)?;
                return a.div(state);
            }
            (Table(a), _) => {
                state.push(other)?;
                return a.div(state);
            }
            (_, _) => {}
        }
        Err(ErrorKind::TypeError(format!("Cannot divide {:?} / {:?}", other, self)).into())
    }
}
