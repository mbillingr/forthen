use crate::errors::Result;
use crate::errors::*;
use crate::rcstring::RcString;
use crate::Object;
use crate::ObjectInterface;
use crate::StackEffect;
use crate::State;
use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;

pub type DynamicObject = Rc<DynamicObjectImpl>;

#[derive(Clone)]
pub struct DynamicObjectImpl {
    attributes: HashMap<RcString, Object>,
}

impl DynamicObjectImpl {
    pub fn new() -> Self {
        DynamicObjectImpl {
            attributes: HashMap::new(),
        }
    }
}

fn invoke_method(this: &DynamicObject, name: &str, state: &mut State) -> Result<()> {
    if let Some(obj) = this.attributes.get(name) {
        state.push(this.clone())?;
        obj.call(state)
    } else {
        this.repr(state)?;
        let r = state.pop_string();
        Err(ErrorKind::AttributeError(format!("no {} method in {:?}", name, r)).into())
    }
}

impl ObjectInterface for DynamicObject {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr_sys(&self) -> String {
        format!("DynamicObject at {:p}", &**self)
    }

    fn repr(&self, state: &mut State) -> Result<()> {
        if let Some(obj) = self.attributes.get("__repr__") {
            state.push(self.clone())?;
            obj.call(state)
        } else {
            state.push_string(self.repr_sys())
        }
    }

    fn cmp_equal(&self, state: &mut State) -> Result<()> {
        invoke_method(self, "__eq__", state)
    }

    fn get_stack_effect(&self) -> Result<&StackEffect> {
        if let Some(obj) = self.attributes.get("__call__") {
            obj.get_stack_effect()
        } else {
            Err(ErrorKind::TypeError(format!("{:?} is not callable", self.repr_sys())).into())
        }
    }
    fn call(&self, state: &mut State) -> Result<()> {
        invoke_method(self, "__call__", state)
    }

    // todo
    fn is_pure(&self) -> bool {
        false
    }

    // todo
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

    fn set_attr(&mut self, attr: Rc<String>, value: Object) {
        Rc::get_mut(self)
            .ok_or(ErrorKind::OwnershipError)
            .unwrap()
            .attributes
            .insert(attr.into(), value);
    }

    fn get_attr(&self, attr: &str) -> Option<Object> {
        self.attributes.get(attr).cloned()
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

    fn add(&self, state: &mut State) -> Result<()> {
        invoke_method(self, "__add__", state)
    }
}

impl From<DynamicObject> for Object {
    fn from(dob: DynamicObject) -> Self {
        Object::Dynamic(dob)
    }
}
