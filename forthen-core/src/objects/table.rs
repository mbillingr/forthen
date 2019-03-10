use crate::errors::Result;
use crate::errors::*;
use crate::rcstring::RcString;
use crate::Object;
use crate::ObjectInterface;
use crate::StackEffect;
use crate::State;
use std::any::Any;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::rc::Rc;

pub type Table = Rc<TableImpl>;

#[derive(Clone, Default)]
pub struct TableImpl {
    attributes: HashMap<RcString, Object>,
    meta: Option<Table>,
}

impl TableImpl {
    pub fn new() -> Self {
        TableImpl {
            attributes: HashMap::new(),
            meta: None,
        }
    }

    pub fn set_metatable(&mut self, meta: Option<Table>) {
        self.meta = meta;
    }

    pub fn get_metatable(&self) -> Option<&Table> {
        self.meta.as_ref()
    }

    fn meta_lookup(&self, attr: &str) -> Option<&Object> {
        self.meta
            .as_ref()
            .and_then(|meta| meta.attributes.get(attr))
    }
}

fn invoke_method(this: &Table, name: &str, state: &mut State) -> Result<()> {
    if let Some(obj) = this.meta_lookup(name) {
        state.push(this.clone())?;
        obj.call(state)
    } else {
        this.repr(state)?;
        let r = state.pop_string().unwrap();
        Err(ErrorKind::AttributeError(format!("no {} method in {:?}", name, r)).into())
    }
}

impl ObjectInterface for Table {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn repr_sys(&self) -> String {
        format!("DynamicObject at {:p}", &**self)
    }

    fn repr(&self, state: &mut State) -> Result<()> {
        if let Some(obj) = self.meta_lookup("__repr__") {
            state.push(self.clone())?;
            obj.call(state)
        } else {
            state.push_string(self.repr_sys())
        }
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

    fn get_attribute(&self, state: &mut State) -> Result<()> {
        let attr: RcString = state.pop()?.try_into_rc_string()?.into();
        if let Some(val) = self.attributes.get(&attr) {
            state.push(val.clone())
        } else if let Some(idx) = self.meta_lookup("__index__") {
            let attr: Rc<String> = attr.into();
            state.push(self.clone())?;
            state.push(Object::String(attr))?;
            idx.call(state)?;
            state.swap()?;
            state.pop()?;
            Ok(())
        } else {
            let attr: &str = attr.borrow();
            Err(ErrorKind::AttributeError(format!(
                "no {} attribute in {:?}",
                attr,
                self.repr_sys()
            ))
            .into())
        }
    }

    fn add(&self, state: &mut State) -> Result<()> {
        invoke_method(self, "__add__", state)
    }

    fn sub(&self, state: &mut State) -> Result<()> {
        invoke_method(self, "__sub__", state)
    }

    fn mul(&self, state: &mut State) -> Result<()> {
        invoke_method(self, "__mul__", state)
    }

    fn div(&self, state: &mut State) -> Result<()> {
        invoke_method(self, "__div__", state)
    }
}

impl From<Table> for Object {
    fn from(dob: Table) -> Self {
        Object::Table(dob)
    }
}
