use super::element::{Element, ElementHash, ElementRef};
use super::parser::parse_effect;
use super::scratchpad::Scratchpad;
use super::sequence::{
    is_sequence_recursive_equivalent, normalized_sequence, sequence_recursive_deepcopy,
};
use crate::errors::*;
use crate::parsing::tokenize;
use crate::stack_effects::astack::AbstractStack;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

#[derive(Default, Clone, PartialEq)]
pub struct StackEffect {
    pub(crate) inputs: Vec<ElementRef>,
    pub(crate) outputs: Vec<ElementRef>,
}

impl StackEffect {
    pub fn new(inputs: Vec<ElementRef>, outputs: Vec<ElementRef>) -> Self {
        StackEffect { inputs, outputs }
    }

    pub fn new_pushing(varname: &str) -> Self {
        let r = ElementRef::anonymous_ellipsis();
        Self::new(
            vec![r.clone()],
            vec![
                r.clone(),
                ElementRef::new(Element::Item(varname.to_string())),
            ],
        )
    }

    pub fn new_quoted(varname: &str, se: StackEffect) -> Self {
        let r = ElementRef::anonymous_ellipsis();
        Self::new(
            vec![r.clone()],
            vec![
                r.clone(),
                ElementRef::new(Element::Callable(varname.to_string(), se)),
            ],
        )
    }

    pub fn new_mod(varname: &str) -> Self {
        let r = ElementRef::anonymous_ellipsis();
        Self::new(
            vec![
                r.clone(),
                ElementRef::new(Element::Item(varname.to_string())),
            ],
            vec![
                r.clone(),
                ElementRef::new(Element::Item(varname.to_string() + "'")),
            ],
        )
    }

    pub fn parse(input: &str) -> Result<Self> {
        let scrpad = &mut Scratchpad::default();
        parse_effect(scrpad, &mut tokenize(input).peekable()).map_err(|e| e)
    }

    pub fn chain(&self, other: &Self) -> Result<StackEffect> {
        let mut astack = AbstractStack::new();
        println!("New abstract stack...");
        println!("applying {}", self);
        astack.apply_effect(self)?;
        println!("applying {}", other);
        astack.apply_effect(other)?;
        dbg!(&astack);
        Ok(dbg!(astack.into_effect()))
    }

    pub fn simplified(self) -> StackEffect {
        let mut inputs = self.inputs;
        let mut outputs = self.outputs;

        loop {
            match (inputs.first(), outputs.first()) {
                (None, _) | (_, None) => break,
                (Some(a), Some(b)) if !a.is_same(b) => break,
                (Some(a), Some(b)) if a.is_same(b) => {
                    if outputs[1..].iter().any(|b| a.is_same(b)) {
                        break;
                    }
                }
                _ => {}
            }

            inputs.remove(0);
            outputs.remove(0);
        }

        StackEffect::new(inputs, outputs)
    }

    pub fn normalized(self) -> Self {
        StackEffect::new(
            normalized_sequence(self.inputs),
            normalized_sequence(self.outputs),
        )
    }

    pub fn is_equivalent(&self, other: &Self) -> bool {
        let mut mapping = HashMap::new();
        self.is_recursive_equivalent(other, &mut mapping)
    }

    pub fn is_recursive_equivalent(
        &self,
        other: &Self,
        mapping: &mut HashMap<usize, usize>,
    ) -> bool {
        is_sequence_recursive_equivalent(&self.inputs, &other.inputs, mapping)
            && is_sequence_recursive_equivalent(&self.outputs, &other.outputs, mapping)
    }

    pub fn recursive_deepcopy(&self, mapping: &mut HashMap<ElementHash, ElementRef>) -> Self {
        let inputs = sequence_recursive_deepcopy(&self.inputs, mapping);
        let outputs = sequence_recursive_deepcopy(&self.outputs, mapping);
        StackEffect::new(inputs, outputs)
    }

    pub fn recursive_display(&self, seen: &mut HashSet<ElementHash>) -> String {
        let simple = self.clone().simplified();

        let a: Vec<_> = simple
            .inputs
            .iter()
            .map(|x| x.recursive_display(seen))
            .collect();
        let b: Vec<_> = simple
            .outputs
            .iter()
            .map(|x| x.recursive_display(seen))
            .collect();

        format!("{} -- {}", a.join(" "), b.join(" "))
    }

    pub fn recursive_dbgstr(&self, seen: &mut HashSet<ElementHash>) -> String {
        let a: Vec<_> = self
            .inputs
            .iter()
            .map(|x| x.recursive_dbgstr(seen))
            .collect();
        let b: Vec<_> = self
            .outputs
            .iter()
            .map(|x| x.recursive_dbgstr(seen))
            .collect();

        format!("StackEffect({} -- {})", a.join(", "), b.join(", "))
    }
}

impl std::fmt::Display for StackEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.recursive_display(&mut HashSet::new()))
    }
}

impl std::fmt::Debug for StackEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.recursive_dbgstr(&mut HashSet::new()))
    }
}

impl FromIterator<StackEffect> for Result<StackEffect> {
    fn from_iter<I: IntoIterator<Item = StackEffect>>(iter: I) -> Self {
        let mut astack = AbstractStack::new();

        for se in iter {
            astack.apply_effect(&se)?;
        }

        Ok(astack.into_effect())
    }
}

impl<'a> FromIterator<&'a StackEffect> for Result<StackEffect> {
    fn from_iter<I: IntoIterator<Item = &'a StackEffect>>(iter: I) -> Self {
        let mut astack = AbstractStack::new();

        for se in iter {
            astack.apply_effect(se)?;
        }

        Ok(astack.into_effect())
    }
}
