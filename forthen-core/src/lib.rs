mod abstract_stack;
mod dictionary;
mod language;
mod object;
mod object_factory;
mod parsing;
mod rcstring;
mod scope;
mod se_interpreter;
mod stack_effect;
mod state;
mod testing;
mod vm;

pub use object::Object;
pub use stack_effect::StackEffect;
pub use state::State;

#[cfg(test)]
mod tests {
    use crate::state::State;

    #[test]
    fn literals() {
        let mut state = State::new();
        state.run("-10 0 25 \"hello forth!\" 2147483647");

        assert_eq!(state.pop_i32(), Some(i32::max_value()));
        assert_eq!(&state.pop_str().unwrap(), "hello forth!");
        assert_eq!(state.pop_i32(), Some(25));
        assert_eq!(state.pop_i32(), Some(0));
        assert_eq!(state.pop_i32(), Some(-10));
    }

    #[test]
    fn it_works() {
    }
}
