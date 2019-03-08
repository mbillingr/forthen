use crate::dictionary::WordId;
use crate::errors::Result;
use crate::objects::{prelude::*, Object};
use crate::stack_effect::StackEffect;
use crate::state::State;

#[derive(Debug, Clone, PartialEq)]
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
            Push(_) => StackEffect::new_pushing("x"),
            Call(obj) => obj.get_stack_effect().clone(),
            TailRecurse => unimplemented!(),
        }
    }
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Opcode::Push(obj) => write!(f, "{:?}", obj),
            Opcode::Call(obj) => write!(f, "{:?}", obj),
            Opcode::TailRecurse => write!(f, "<tail recurse>"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ByteCode {
    pub ops: Vec<Opcode>,
}

impl ByteCode {
    pub fn new() -> Self {
        ByteCode { ops: vec![] }
    }

    pub fn run(&self, state: &mut State) -> Result<()> {
        use Opcode::*;
        'outer: loop {
            for op in &self.ops {
                match op {
                    Push(obj) => state.push(obj.clone())?,
                    Call(obj) => obj.call(state)?,
                    TailRecurse => continue 'outer,
                }
            }
            break;
        }
        Ok(())
    }
}

impl std::fmt::Display for ByteCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let items: Vec<_> = self.ops.iter().map(|op| format!("{}", op)).collect();
        write!(f, "{}", items.join(" "))
    }
}
