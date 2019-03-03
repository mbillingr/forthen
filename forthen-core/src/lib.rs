mod abstract_stack;
mod dictionary;
pub mod error;
mod object;
mod object_factory;
mod parsing;
mod rcstring;
mod scope;
mod stack_effect;
mod state;
mod testing;
mod vm;

pub use error::{Error, Result};
pub use object::{DynamicObject, Object};
pub use scope::CompilerScope;
pub use stack_effect::StackEffect;
pub use state::State;
pub use vm::{Opcode, Quotation};

#[cfg(test)]
mod tests {
    use crate::state::State;

    #[test]
    fn literals() {
        let mut state = State::new();
        state.run("-10 0 25 \"hello forth!\" 2147483647").unwrap();

        assert_eq!(state.pop_i32(), Ok(i32::max_value()));
        assert_eq!(&state.pop_str().unwrap(), "hello forth!");
        assert_eq!(state.pop_i32(), Ok(25));
        assert_eq!(state.pop_i32(), Ok(0));
        assert_eq!(state.pop_i32(), Ok(-10));
    }

    #[test]
    fn it_works() {}
}
