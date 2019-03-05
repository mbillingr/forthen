use crate::errors::Result;
use crate::State;
use crate::Object;
use std::rc::Rc;

/// A pure function has no side effects. It can only access the value stack.
pub type PureFunction = fn(&mut Vec<Object>) -> Result<()>;

/// A native function can manipulate the entire state.
pub type NativeFunction = fn(&mut State) -> Result<()>;

/// A closure is allowed to contain external state.
pub type NativeClosure = Rc<dyn Fn(&mut State) -> Result<()>>;
