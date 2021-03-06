#[macro_use]
extern crate error_chain;

mod dictionary;
pub mod errors;
mod module;
pub mod object_factory;
pub mod objects;
mod parsing;
mod rcstring;
mod refhash;
mod scope;
mod stack_effects;
mod state;
mod testing;
mod vm;

pub use errors::{Error, Result};
pub use objects::prelude::*;
pub use scope::CompilerScope;
pub use stack_effects::{IntoStackEffect, StackEffect};
pub use state::{Mode, State};
pub use vm::{ByteCode, Opcode};

#[cfg(test)]
mod tests {
    use crate::state::State;

    #[test]
    fn literals() {
        let mut state = State::new();
        state.run("-10 0 25 \"hello forth!\" 2147483647").unwrap();

        assert_eq!(state.pop_i32().unwrap(), i32::max_value());
        assert_eq!(&state.pop_string().unwrap(), "hello forth!");
        assert_eq!(state.pop_i32().unwrap(), 25);
        assert_eq!(state.pop_i32().unwrap(), 0);
        assert_eq!(state.pop_i32().unwrap(), -10);
    }
}
