pub use super::Object;
use crate::Result;
use crate::StackEffect;
use crate::State;
use std::any::Any;

pub trait DynamicObject {
    fn as_any(&self) -> &dyn Any;
    fn repr(&self) -> String;
    fn eq(&self, other: &dyn DynamicObject) -> bool;
}

// WIP below

pub trait ObjectInterface {
    fn as_any(&self) -> &dyn Any;
    fn as_number(&self) -> Option<&dyn NumberInterface>;
    fn as_callable(&self) -> Option<&dyn NumberInterface>;
    fn as_sequence(&self) -> Option<&dyn SequenceInterface>;

    fn repr(&self, _state: &mut State) -> Result<()>;

    fn cmp_equal(&self, _state: &mut State) -> Result<()>;

    fn is_number(&self) -> bool {
        self.as_number().is_some()
    }

    fn is_callable(&self) -> bool {
        self.as_callable().is_some()
    }

    fn is_sequence(&self) -> bool {
        self.as_sequence().is_some()
    }
}

pub trait NumberInterface {
    fn add(&self, _state: &mut State) -> Result<()>;

    fn push_integer(&self, _state: &mut State) -> Result<()>;
    fn push_float(&self, _state: &mut State) -> Result<()>;
}

pub trait CallableInterface {
    fn get_stack_effect(&self) -> &StackEffect;
    fn call(&self, _state: &mut State) -> Result<()>;
    fn is_pure(&self) -> bool;
}

pub trait SequenceInterface {
    fn as_vec_mut(&mut self) -> Result<&mut Vec<Object>>;
    fn as_slice(&self) -> Result<&[Object]>;
}
