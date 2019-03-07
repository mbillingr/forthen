use crate::errors::Result;
use crate::Object;
use crate::StackEffect;
use crate::State;
use std::cell::RefCell;
use std::rc::Rc;
use super::prelude::*;

type Pure = fn(&mut Vec<Object>) -> Result<()>;
type Const = dyn Fn(&mut State) -> Result<()>;
type Mutie = RefCell<dyn FnMut(&mut State) -> Result<()>>;

// todo: How can we hide (make private) this implementation detail?
pub struct WithStackEffect<F: ?Sized> {
    se: StackEffect,
    func: F,
}

#[derive(Clone)]
pub enum Callable {
    Pure(Pure, Rc<StackEffect>),
    Const(Rc<WithStackEffect<Const>>),
    Mutie(Rc<WithStackEffect<Mutie>>),
}

impl std::fmt::Debug for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Callable::Pure(func, _) => write!(f, "pure fn @ {:p}", &func),
            Callable::Const(ca) => write!(f, "const fn @ {:p}", &ca.func),
            Callable::Mutie(ca) => write!(f, "mutie fn @ {:p}", &*ca.func.borrow()),
        }
    }
}

impl Callable {
    pub fn new_pure(func: Pure, se: StackEffect) -> Self {
        Callable::Pure(func, Rc::new(se))
    }

    pub fn new_const<F: 'static + Fn(&mut State) -> Result<()>>(func: F, se: StackEffect) -> Self {
        Callable::Const(Rc::new(WithStackEffect { se, func }))
    }

    pub fn new_mutie<F: 'static + FnMut(&mut State) -> Result<()>>(
        func: F,
        se: StackEffect,
    ) -> Self {
        Callable::Mutie(Rc::new(WithStackEffect {
            se,
            func: RefCell::new(func),
        }))
    }

    fn as_ptr(&self) -> *const u8 {
        match self {
            Callable::Pure(func, _) => func as *const _ as *const u8,
            Callable::Const(ca) => &ca.func as *const _ as *const u8,
            Callable::Mutie(ca) => &*ca.func.borrow() as *const _ as *const u8,
        }
    }
}

impl CallableInterface for Callable {
    fn get_stack_effect(&self) -> &StackEffect {
        match self {
            Callable::Pure(_, se) => &*se,
            Callable::Const(ca) => &ca.se,
            Callable::Mutie(ca) => &ca.se,
        }
    }
    fn call(&self, state: &mut State) -> Result<()> {
        match self {
            Callable::Pure(f, _) => f(&mut state.stack),
            Callable::Const(ca) => (ca.func)(state),
            Callable::Mutie(ca) => (&mut *ca.func.borrow_mut())(state),
        }
    }

    fn is_pure(&self) -> bool {
        match self {
            Callable::Pure(_, _) => true,
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