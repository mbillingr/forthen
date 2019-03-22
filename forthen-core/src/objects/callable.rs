use crate::errors::Result;
use crate::Object;
use crate::State;
use std::cell::RefCell;
use std::rc::Rc;

type Pure = fn(&mut Vec<Object>) -> Result<()>;
type Const = dyn Fn(&mut State) -> Result<()>;
type Mutie = RefCell<dyn FnMut(&mut State) -> Result<()>>;

#[derive(Clone)]
pub enum Callable {
    Pure(Pure),
    Const(Rc<Const>),
    Mutie(Rc<Mutie>),
}

impl std::fmt::Debug for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Callable::Pure(func) => write!(f, "pure fn @ {:p}", &func),
            Callable::Const(func) => write!(f, "const fn @ {:p}", &func),
            Callable::Mutie(func) => write!(f, "mutie fn @ {:p}", &*func.borrow()),
        }
    }
}

impl Callable {
    pub fn new_pure(func: Pure) -> Self {
        Callable::Pure(func)
    }

    pub fn new_const<F: 'static + Fn(&mut State) -> Result<()>>(func: F) -> Self {
        Callable::Const(Rc::new(func))
    }

    pub fn new_mutie<F: 'static + FnMut(&mut State) -> Result<()>>(func: F) -> Self {
        Callable::Mutie(Rc::new(RefCell::new(func)))
    }

    fn as_ptr(&self) -> *const u8 {
        match self {
            Callable::Pure(func) => func as *const _ as *const u8,
            Callable::Const(func) => &func as *const _ as *const u8,
            Callable::Mutie(func) => &*func.borrow() as *const _ as *const u8,
        }
    }

    pub fn call(&self, state: &mut State) -> Result<()> {
        match self {
            Callable::Pure(f) => f(&mut state.stack),
            Callable::Const(f) => f(state),
            Callable::Mutie(f) => (&mut *f.borrow_mut())(state),
        }
    }

    pub fn is_pure(&self) -> bool {
        match self {
            Callable::Pure(_) => true,
            Callable::Const(_) => false,
            Callable::Mutie(_) => false,
        }
    }
}

impl std::cmp::PartialEq for Callable {
    fn eq(&self, other: &Self) -> bool {
        self.as_ptr() == other.as_ptr()
    }
}
