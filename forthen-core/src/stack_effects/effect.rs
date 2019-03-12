use crate::errors::*;
use crate::parsing::tokenize;
use super::comparison::is_sequence_equivalent;
use super::element::ElementRef;
use super::parser::parse_effect;
use super::scratchpad::Scratchpad;
use super::sequence::normalized_sequence;
use std::collections::HashSet;

#[derive(Default, Clone, PartialEq)]
pub struct StackEffect {
    inputs: Vec<ElementRef>,
    outputs: Vec<ElementRef>,
}

impl StackEffect {
    pub fn new(inputs: Vec<ElementRef>, outputs: Vec<ElementRef>) -> Self {
        StackEffect {inputs, outputs}
    }

    pub fn parse(input: &str) -> Result<Self> {
        let scrpad = &mut Scratchpad::default();
        parse_effect(scrpad, &mut tokenize(input).peekable()).map_err(|e| e)
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
        StackEffect::new(normalized_sequence(self.inputs), normalized_sequence(self.outputs))
    }

    pub fn is_equivalent(&self, other: &Self) -> bool {
        is_sequence_equivalent(&self.inputs, &other.inputs) && is_sequence_equivalent(&self.outputs, &other.outputs)
    }

    pub fn recursive_display(&self, seen: &mut HashSet<String>) -> String {
        let simple = self.clone().simplified();

        let a: Vec<_> = simple.inputs.iter().map(|x| x.borrow().recursive_display(seen)).collect();
        let b: Vec<_> = simple.outputs.iter().map(|x| x.borrow().recursive_display(seen)).collect();

        format!("{} -- {}", a.join(" "), b.join(" "))
    }

    pub fn recursive_dbgstr(&self, seen: &mut HashSet<String>) -> String {

        let a: Vec<_> = self.inputs.iter().map(|x| x.borrow().recursive_dbgstr(seen)).collect();
        let b: Vec<_> = self.outputs.iter().map(|x| x.borrow().recursive_dbgstr(seen)).collect();

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
