pub use super::Object;
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

    fn is_number(&self) -> bool {
        self.as_number().is_some()
    }
}

pub trait NumberInterface {}

pub trait CallableInterface {}
