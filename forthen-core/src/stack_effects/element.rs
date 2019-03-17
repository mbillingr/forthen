use super::effect::StackEffect;
use super::sequence::sequence_recursive_deepcopy;
use crate::errors::*;
use crate::refhash::RefHash;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use crate::stack_effects::sequence::normalized_sequence;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ElementHash(RefHash<RefCell<Element>>);

impl From<ElementRef> for ElementHash {
    fn from(er: ElementRef) -> Self {
        ElementHash(RefHash::new(er.node))
    }
}

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

    pub fn anonymous_ellipsis() -> Self {
        Self::new(Element::Ellipsis("".into()))
    }

    pub fn addr(&self) -> usize {
        &*self.node as *const _ as usize
    }

    pub fn borrow(&self) -> impl Deref<Target = Element> + '_ {
        self.node.borrow()
    }

    pub fn borrow_mut(&self) -> impl DerefMut<Target = Element> + '_ {
        self.node.borrow_mut()
    }

    pub fn try_borrow_mut(&self) -> Option<impl DerefMut<Target = Element> + '_> {
        self.node.try_borrow_mut().ok()
    }

    pub fn is_same(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.node, &other.node)
    }

    pub fn flattened(self) -> Self {
        if let Element::Sequence(s) = &*self.borrow() {
            if s.len() == 1 {
                return s[0].clone().flattened()
            }
        }
        self
    }

    pub fn substitute(&self, mut new_content: Element) -> Element {

        if let Element::Sequence(ref s) = new_content {
            if s.len() == 1 && s[0].is_same(self) {
                return new_content
            }
        }

        let old = std::mem::replace(&mut *self.borrow_mut(), new_content);
        old
    }

    pub fn recursive_deepcopy(&self, mapping: &mut HashMap<ElementHash, ElementRef>) -> Self {
        let eh = self.clone().into();
        if let Some(y) = mapping.get(&eh) {
            return y.clone();
        }

        let new_el = match &*self.borrow() {
            Element::Ellipsis(name) => Element::Ellipsis(name.clone()),
            Element::Item(name) => Element::Item(name.clone()),
            Element::Callable(name, se) => {
                Element::Callable(name.clone(), se.recursive_deepcopy(mapping))
            }
            Element::Sequence(seq) => Element::Sequence(sequence_recursive_deepcopy(seq, mapping)),
        };

        let y = Self::new(new_el);
        mapping.insert(eh, y.clone());
        y
    }

    pub fn recursive_display(&self, seen: &mut HashSet<ElementHash>) -> String {
        match &*self.borrow() {
            Element::Ellipsis(name) => format!("..{}", name),
            Element::Item(name) => name.to_string(),
            Element::Callable(name, se) => {
                if seen.contains(&self.clone().into()) {
                    format!("{}(...)", name)
                } else {
                    seen.insert(self.clone().into());
                    format!("{}({})", name, se.recursive_display(seen))
                }
            }
            Element::Sequence(elements) => elements
                .iter()
                .map(|ele| ele.recursive_display(seen))
                .collect::<Vec<_>>()
                .join(" "),
        }
    }

    pub fn recursive_dbgstr(&self, seen: &mut HashSet<ElementHash>) -> String {
        match &*self.borrow() {
            Element::Ellipsis(name) => format!("Ellipsis({:?})", name),
            Element::Item(name) => format!("Item({:?})", name),
            Element::Callable(name, se) => {
                if seen.contains(&self.clone().into()) {
                    format!("Callable({:?}, ...)", name)
                } else {
                    seen.insert(self.clone().into());
                    format!("Callable({}, {})", name, se.recursive_dbgstr(seen))
                }
            }
            Element::Sequence(elements) => format!("[{}]", elements
                .iter()
                .map(|ele| ele.recursive_dbgstr(seen))
                .collect::<Vec<_>>()
                .join(" ")),
        }
    }
}

impl std::fmt::Display for ElementRef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.recursive_display(&mut HashSet::new()))
    }
}

impl std::fmt::Debug for ElementRef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.recursive_dbgstr(&mut HashSet::new()))
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
    Sequence(Vec<ElementRef>),
}

impl Element {
    pub fn flattened(self) -> Self {
        if let Element::Sequence(s) = self {
            Element::Sequence(normalized_sequence(s))
        } else {
            self
        }
    }

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

    pub fn name(&self) -> Option<&str> {
        match self {
            Element::Ellipsis(name) | Element::Item(name) | Element::Callable(name, _) => {
                Some(name)
            }
            Element::Sequence(_) => None,
        }
    }

    /// Put the more specific item in self and return the other.
    /// Return an error if items are not compatible.
    pub fn replace_if_more_specific(&mut self, mut other: Self) -> Result<Self> {
        use Element::*;
        match (&self, &mut other) {
            (_, Ellipsis(_)) => {}
            (Ellipsis(_), _) => std::mem::swap(self, &mut other),
            (_, Item(_)) => {}
            (Item(_), _) => std::mem::swap(self, &mut other),
            (Callable(_, _), Callable(_, _)) => unimplemented!(),
            (Sequence(_), Sequence(_)) => unimplemented!(),
            (Callable(_, _), Sequence(_)) => return Err(ErrorKind::IncompatibleStackEffects.into()),
            (Sequence(_), Callable(_, _)) => return Err(ErrorKind::IncompatibleStackEffects.into()),
        }
        Ok(other)
    }

    /// return true if self is less specific than other
    pub fn is_less_specific(&self, other: &Self) -> Result<bool> {
        use Element::*;
        match (self, other) {
            (Ellipsis(_), Ellipsis(_)) => Ok(false),
            (Item(_), Item(_)) => Ok(false),
            (Callable(_, _), Callable(_, _)) => Ok(false),
            (Sequence(_), Sequence(_)) => Ok(false),
            (Callable(_, _), Sequence(_)) => Err(ErrorKind::IncompatibleStackEffects.into()),
            (Sequence(_), Callable(_, _)) => Err(ErrorKind::IncompatibleStackEffects.into()),
            // note the order of the ones below...
            (_, Ellipsis(_)) => Ok(false),
            (Ellipsis(_), _) => Ok(true),
            (_, Item(_)) => Ok(false),
            (Item(_), _) => Ok(true),
        }
    }
}
/*
impl std::fmt::Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.recursive_display(&mut HashSet::new()))
    }
}

impl std::fmt::Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.recursive_dbgstr(&mut HashSet::new()))
    }
}*/
