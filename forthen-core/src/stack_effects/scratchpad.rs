use super::effect::StackEffect;
use super::element::{Element, ElementRef};
use crate::errors::*;
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct Scratchpad {
    elements: Vec<ElementRef>,
}

impl Scratchpad {
    pub fn update(&mut self, new_node: Element) -> ElementRef {
        for noderef in &self.elements {
            if noderef.borrow().name() == new_node.name() {
                noderef
                    .borrow_mut()
                    .replace_if_more_specific(new_node)
                    .unwrap();
                return noderef.clone();
            }
        }
        self.insert(new_node)
    }

    pub fn insert(&mut self, new_node: Element) -> ElementRef {
        let r = ElementRef::new(new_node);
        self.elements.push(r.clone());
        r
    }

    pub fn insert_existing(&mut self, node: ElementRef) {
        self.elements.push(node);
    }

    pub fn copy_effect(&mut self, se: &StackEffect) -> StackEffect {
        let mut mapping = HashMap::new();
        let new_se = se.recursive_deepcopy(&mut mapping);
        self.elements.extend(mapping.into_iter().map(|(_, v)| v));
        new_se
    }

    /// replace the more generic element with the more specific one and return the result
    pub fn substitute(&mut self, a: ElementRef, b: ElementRef) -> Result<ElementRef> {
        if a.borrow().is_less_specific(&b.borrow())? {
            a.substitute(Element::Sequence(vec![b]));
            Ok(a)
        } else {
            b.substitute(Element::Sequence(vec![a]));
            Ok(b)
        }
    }
}

#[cfg(test)]
impl Scratchpad {
    pub fn find_by_name(&self, name: &str) -> Option<&ElementRef> {
        self.elements
            .iter()
            .find(|e| e.borrow().name() == Some(name))
    }
}
