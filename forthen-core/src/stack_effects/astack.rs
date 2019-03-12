use super::scratchpad::Scratchpad;
use super::element::{Element, ElementRef};
use super::effect::StackEffect;
use super::sequence::normalized_sequence;
use crate::errors::*;
use std::sync::atomic::Ordering::SeqCst;
use std::io::SeekFrom::Start;

#[derive(Debug, Clone)]
pub struct AbstractStack {
    scratchpad: Scratchpad,

    inputs: Vec<ElementRef>,
    outputs: Vec<ElementRef>,
}

impl AbstractStack {
    pub fn new() -> Self {
        let mut scratchpad = Scratchpad::default();
        let r = ElementRef::anonymous_ellipsis();
        scratchpad.insert_existing(r.clone());
        AbstractStack {
            scratchpad,
            inputs: vec![r.clone()],
            outputs: vec![r],
        }
    }

    pub fn apply_effect(&mut self, se: &StackEffect) -> Result<()> {
        let StackEffect{inputs, outputs} = self.scratchpad.copy_effect(se);

        for i in inputs.into_iter().rev() {
            self.pop(i)?;
        }

        for o in outputs {
            self.push(o);
        }

        self.inputs = normalized_sequence(self.inputs.clone());
        self.outputs = normalized_sequence(self.outputs.clone());

        Ok(())
    }

    pub fn push(&mut self, elem: ElementRef) {
        self.outputs.push(elem);
    }

    pub fn pop(&mut self, elem: ElementRef) -> Result<(ElementRef)> {
        if elem.borrow().is_ellipsis() {
            let x = std::mem::replace(&mut self.outputs, vec![]);
            elem.substitute(Element::Sequence(x));
            return Ok(elem)
        }

        match self.outputs.pop() {
            None => panic!("Abstract Stack Underflow"),
            Some(x) =>{
                if x.borrow().is_ellipsis() {
                    self.outputs.push(x.clone());
                    self.inputs.insert(1, elem.clone());
                    Ok(elem)
                } else {
                    self.scratchpad.substitute(x, elem)
                }
            }
        }
    }

    pub fn into_effect(self) -> StackEffect {
        StackEffect::new(self.inputs, self.outputs).normalized()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abstract_stack() {
        let mut astack = AbstractStack::new();

        let swap = StackEffect::parse("(a b -- b a)").unwrap();
        let nop = StackEffect::parse("(a b -- a b)").unwrap();

        astack.apply_effect(&swap).unwrap();
        assert!(astack.clone().into_effect().is_equivalent(&swap));

        astack.apply_effect(&swap).unwrap();
        assert!(astack.clone().into_effect().is_equivalent(&nop));
    }

    #[test]
    fn complex_chain() {
        let mut astack = AbstractStack::new();

        let dup = StackEffect::parse("(x -- x x)").unwrap();
        let drop = StackEffect::parse("(z -- )").unwrap();
        let swap = StackEffect::parse("(a b -- b a)").unwrap();

        astack.apply_effect(&dup).unwrap();
        astack.apply_effect(&drop).unwrap();
        astack.apply_effect(&swap).unwrap();
    }

}