mod dictionary;
mod language;
mod object;
mod object_factory;
mod parsing;
mod rcstring;
mod stack_effect;
mod state;
mod testing;

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
        let mut state = State::new();
        state.tier0();
        state.tier1();

        state.run("3 5 \"hello forth!\" .s");
        state.run("3 5 \"hello forth!\" .s");

        state.run(": the-answer 42 ;");
        state.run("the-answer .s");

        println!("{:#?}", state);

        panic!("panicking so we can see the output :)");
    }
}
