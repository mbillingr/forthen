use super::effect::StackEffect;
use std::cell::{Ref, RefCell};
use std::ops::Deref;
use std::rc::Rc;

#[derive(Clone)]
pub struct ElementRef {
    node: Rc<RefCell<Element>>,
}

impl ElementRef {
    pub fn new(el: Element) -> Self {
        ElementRef {
            node: Rc::new(RefCell::new(el)),
        }
    }

    pub fn borrow(&self) -> impl Deref<Target = Element> + '_ {
        self.node.borrow()
    }

    pub fn is_same(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.node, &other.node)
    }
}

impl std::fmt::Debug for ElementRef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.node.borrow().fmt(f)
    }
}

impl std::cmp::PartialEq for ElementRef {
    fn eq(&self, rhs: &Self) -> bool {
        self.is_same(rhs)
    }
}

#[derive(Clone)]
pub enum Element {
    Ellipsis(String),
    Item(String),
    Callable(String, StackEffect),
}

impl Element {
    pub fn is_same(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }

    pub fn is_ellipsis(&self) -> bool {
        if let Element::Ellipsis(_) = self {
            true
        } else {
            false
        }
    }

    pub fn simplified(&self) -> Self {
        match self {
            Element::Ellipsis(_) | Element::Item(_) => self.clone(),
            Element::Callable(name, se) => Element::Callable(name.clone(), se.simplified()),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Element::Ellipsis(name) | Element::Item(name) | Element::Callable(name, _) => name,
        }
    }
}

impl std::fmt::Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.simplified() {
            Element::Ellipsis(name) => write!(f, "..{}", name),
            Element::Item(name) => write!(f, "{}", name),
            Element::Callable(name, se) => write!(f, "{}({})", name, se),
        }
    }
}
