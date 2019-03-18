use super::effect::StackEffect;
use super::element::{Element, ElementRef};
use crate::errors::*;
use std::collections::HashMap;
use std::hint::unreachable_unchecked;
use crate::stack_effects::sequence::normalized_sequence;

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
        use Element::*;

        println!("substituting items:\n    {:?}\n    {:?}", a, b);

        if b.borrow().is_less_specific(&a.borrow())? {
            return self.substitute(b, a)
        }

        match (&mut *a.borrow_mut(), &mut *b.borrow_mut()) {
            (Callable(_, ref mut sea), Callable(_, ref mut seb)) => {
                sea.inputs = normalized_sequence(sea.inputs.clone());
                sea.outputs = normalized_sequence(sea.outputs.clone());
                seb.inputs = normalized_sequence(seb.inputs.clone());
                seb.outputs = normalized_sequence(seb.outputs.clone());
                self.substitute_sequences(sea.inputs.clone(), seb.inputs.clone())?;
                self.substitute_sequences(sea.outputs.clone(), seb.outputs.clone())?;
            }
            (Sequence(sa), Sequence(sb)) => {
                panic!();
                self.substitute_sequences(sa.clone(), sb.clone())?;
            }
            _ => {}
        }

        a.substitute(Element::Sequence(vec![b]))?;
        Ok(a)
    }

    fn substitute_sequences(&mut self, a: Vec<ElementRef>, b: Vec<ElementRef>) -> Result<()> {
        use Element::*;

        let a = normalized_sequence(a);
        let b = normalized_sequence(b);

        println!("substituting sequences:\n    {:?}\n    {:?}", a, b);

        match (a.len(), b.len()) {
            (0, 0) => return Ok(()),
            (0, _) | (_, 0) => unreachable!("Internal substitution error"),
            _ => {}
        }

        let (a_left, a_right) = a.split_at(a.len() - 1);
        let (b_left, b_right) = b.split_at(b.len() - 1);

        println!("{:?} {:?} : {:?} {:?}", a_left, a_right, b_left, b_right);

        if !a_right[0].is_same(&b_right[0]) {
            println!("running {:?} <-> {:?}", a_right[0], b_right[0]);

            if a_right[0].borrow().is_ellipsis() {
                a_right[0].substitute(Sequence(b.to_vec()).flattened())?;
                println!("OK");
                return Ok(())
            }

            if b_right[0].borrow().is_ellipsis() {
                println!("{:?}", a.to_vec());
                b_right[0].substitute(Sequence(a.to_vec()).flattened())?;
                println!("OK");
                return Ok(())
            }
            println!("..");

            self.substitute(a_right[0].clone(), b_right[0].clone())?;
            println!("OK");
        }

        self.substitute_sequences(a_left.to_vec(), b_left.to_vec())
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
