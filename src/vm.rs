use crate::dictionary::WordId;
use crate::object::Object;
use crate::stack_effect::StackEffect;
use crate::state::State;

#[derive(Debug, PartialEq)]
pub enum Opcode {
    Push(Object),
    Call(Object),
    TailRecurse,
}

impl Opcode {
    pub fn push_i32(i: i32) -> Self {
        Opcode::Push(Object::I32(i))
    }

    pub fn call_word(id: WordId) -> Self {
        Opcode::Call(Object::Word(id))
    }

    pub fn stack_effect(&self) -> StackEffect {
        use Opcode::*;
        match self {
            Push(Object::Quotation(_, se)) => StackEffect::new_quotation("f", se.clone()),
            Push(_) => StackEffect::new_pushing("x"),
            Call(obj) => obj.get_stack_effect(),
            TailRecurse => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Quotation {
    pub ops: Vec<Opcode>,
    //se: StackEffect,
}

impl Quotation {
    pub fn new() -> Self {
        Quotation { ops: vec![] }
    }

    pub fn run(&self, state: &mut State) {
        use Opcode::*;
        'outer: loop {
            for op in &self.ops {
                match op {
                    Push(obj) => state.push(obj.clone()),
                    Call(obj) => obj.invoke(state),
                    TailRecurse => continue 'outer,
                }
            }
            break;
        }
    }
}
