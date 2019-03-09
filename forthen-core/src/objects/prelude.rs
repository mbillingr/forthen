use super::table::Table;
pub use super::Object;
use crate::errors::*;
use crate::Result;
use crate::StackEffect;
use crate::State;
use std::any::Any;
use std::rc::Rc;

pub trait ObjectInterface {
    fn as_any(&self) -> &dyn Any;

    fn repr_sys(&self) -> String;

    fn repr(&self, state: &mut State) -> Result<()> {
        state.push_string(self.repr_sys())
    }

    fn cmp_equal(&self, _state: &mut State) -> Result<()> {
        Err(ErrorKind::TypeError(format!(
            "cmp_equal not implemented for {:?}",
            self.repr_sys()
        ))
        .into())
    }

    fn is_number(&self) -> bool {
        false
    }
    fn is_callable(&self) -> bool {
        false
    }
    fn is_sequence(&self) -> bool {
        false
    }

    fn get_stack_effect(&self) -> Result<&StackEffect> {
        Err(
            ErrorKind::TypeError(format!("{:?} does not have stack effects", self.repr_sys()))
                .into(),
        )
    }
    fn call(&self, _state: &mut State) -> Result<()> {
        Err(ErrorKind::TypeError(format!("{:?} is not callable", self.repr_sys())).into())
    }
    fn is_pure(&self) -> bool {
        false
    }

    fn as_vec_mut(&mut self) -> Result<&mut Vec<Object>> {
        Err(ErrorKind::TypeError(format!(
            "as_vec_mut not implemented for {:?}",
            self.repr_sys()
        ))
        .into())
    }

    fn as_slice(&self) -> Result<&[Object]> {
        Err(ErrorKind::TypeError(format!(
            "as_slice not implemented for {:?}",
            self.repr_sys()
        ))
        .into())
    }

    fn set_meta(&mut self, _meta: Option<Table>) -> Result<()> {
        Err(ErrorKind::TypeError(format!(
            "get/set metatable not implemented for {:?}",
            self.repr_sys()
        ))
        .into())
    }

    fn get_meta(&mut self) -> Option<Table> {
        None
    }

    fn set_attr(&mut self, _attr: Rc<String>, _value: Object) {
        panic!("set attribute not implemented for {:?}", self.repr_sys())
    }

    fn get_attr(&self, _attr: &str) -> Option<Object> {
        None
    }

    fn set_attribute(&mut self, _state: &mut State) -> Result<()> {
        Err(ErrorKind::TypeError(format!(
            "get/set attribute not implemented for {:?}",
            self.repr_sys()
        ))
        .into())
    }
    fn get_attribute(&mut self, _state: &mut State) -> Result<()> {
        Err(ErrorKind::TypeError(format!(
            "get/set attribute not implemented for {:?}",
            self.repr_sys()
        ))
        .into())
    }

    fn add(&self, _state: &mut State) -> Result<()> {
        Err(ErrorKind::TypeError(format!("add not implemented for {:?}", self.repr_sys())).into())
    }
}
