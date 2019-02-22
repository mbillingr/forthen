use crate::stack_effect::StackEffect;
use crate::vm::{Opcode, Quotation};

fn stack_effect_interpreter(quot: &Quotation) -> StackEffect {
    let mut astack = AbstractEffectStack::new();

    /*for op in &quot.ops {
        match op {
            Opcode::Push(obj) => astack.push(obj.try_)
        }
        se = se.chain(op.stack_effect());
    }*/

    unimplemented!()
}

struct AbstractEffectStack {}

impl AbstractEffectStack {
    fn new() -> Self {
        AbstractEffectStack {}
    }
}
