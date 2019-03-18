use super::effect::StackEffect;
use super::element::{Element, ElementRef};
use super::scratchpad::Scratchpad;
use super::sequence::normalized_sequence;
use crate::errors::*;

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
        let StackEffect { inputs, outputs } = self.scratchpad.copy_effect(se);

        for i in inputs.into_iter().rev() {
            self.pop(i)?;
        }

        for o in outputs {
            self.push(o);
        }

        self.inputs = normalized_sequence(self.inputs.clone());
        self.outputs = normalized_sequence(self.outputs.clone());

        for (k, (i, o)) in self.inputs.iter_mut().zip(&mut self.outputs).enumerate() {
            if !i.is_same(o) {
                break
            }
            
            if k == 0 { continue }

            if i.borrow().is_ellipsis() {
                *i = ElementRef::new(Element::Sequence(vec![]));
                *o = ElementRef::new(Element::Sequence(vec![]));
            }
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
            elem.substitute(Element::Sequence(x))?;
            return Ok(elem);
        }

        match self.outputs.pop() {
            None => panic!("Abstract Stack Underflow"),
            Some(x) => {
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
    use super::super::IntoStackEffect;

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

    #[test]
    fn if_chain() {
        let sfx = "(..a ? yes(..a -- ..b) no(..a -- ..b) -- ..b)".into_stack_effect();
        let yes = "(..d -- ..d f(..c -- ..c y))".into_stack_effect();
        let no = "(..d -- ..d f(..c -- ..c n))".into_stack_effect();
        let put = "(..d -- ..d f(..c -- ..c p))".into_stack_effect();
        let drop = "(..d -- ..d f(..c x -- ..c))".into_stack_effect();

        assert!(yes.chain(&no).unwrap().chain(&sfx).unwrap().is_equivalent(&"(cond -- value)".into_stack_effect()));
        assert!(drop.chain(&drop).unwrap().chain(&sfx).unwrap().is_equivalent(&"(x ? -- )".into_stack_effect()));
        assert!(put.chain(&put).unwrap().chain(&sfx).unwrap().is_equivalent(&"(? -- x)".into_stack_effect()));

        if let Ok(_) = put.chain(&drop).unwrap().chain(&sfx) {
            panic!("Expected Error")
        }

        if let Ok(_) = drop.chain(&put).unwrap().chain(&sfx) {
            panic!("Expected Error")
        }
    }
}
