use super::element::ElementRef;
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

    pub fn simplified(&self) -> StackEffect {
        let mut inputs = self.inputs.clone();
        let mut outputs = self.outputs.clone();

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

        StackEffect { inputs, outputs }
    }

    pub fn recursive_display(&self, seen: &mut HashSet<String>) -> String {
        let simple = self.simplified();

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
