use crate::Result;
use crate::State;
use std::rc::Rc;

pub type NativeFunction = fn(&mut State) -> Result<()>;
pub type NativeClosure = Rc<dyn Fn(&mut State) -> Result<()>>;
