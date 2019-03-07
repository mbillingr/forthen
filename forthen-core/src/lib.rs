#[macro_use]
extern crate error_chain;

mod abstract_stack;
mod dictionary;
pub mod errors;
mod object_factory;
pub mod objects;
mod parsing;
mod rcstring;
mod scope;
mod stack_effect;
mod state;
mod testing;
mod vm;

pub use errors::{Error, Result};
pub use objects::prelude::*;
pub use scope::CompilerScope;
pub use stack_effect::StackEffect;
pub use state::State;
pub use vm::{Opcode, ByteCode};

#[cfg(test)]
mod tests {
    use crate::state::State;

    #[test]
    fn literals() {
        let mut state = State::new();
        state.run("-10 0 25 \"hello forth!\" 2147483647").unwrap();

        assert_eq!(state.pop_i32().unwrap(), i32::max_value());
        assert_eq!(&state.pop_str().unwrap(), "hello forth!");
        assert_eq!(state.pop_i32().unwrap(), 25);
        assert_eq!(state.pop_i32().unwrap(), 0);
        assert_eq!(state.pop_i32().unwrap(), -10);
    }

    #[test]
    fn it_works() {}
}
