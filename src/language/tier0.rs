use crate::state::State;

impl State {
    /// Load language tier 0 into the dictionary
    ///
    /// Tier 0 contains low level native words required for extending the language
    pub fn tier0(&mut self) {
        self.add_native_parse_word(":", |state| {
            let name = state.next_token().expect("word name");
            state.begin_compile();
            state.parse_until(";");
            let ops = state.pop();
            state.add_compound_word(name, ops.into_rc_vec());
        });

        self.add_native_parse_word("SYNTAX:", |state| {
            let name = state.next_token().expect("word name");
            state.begin_compile();
            state.parse_until(";");
            let ops = state.pop();
            state.add_compound_parse_word(name, ops.into_rc_vec());
        });

        self.add_native_parse_word(";", |_| panic!("Parse Error"));
    }
}

#[cfg(test)]
mod tests {
    use super::State;

    #[test]
    fn new_words() {
        let mut state = State::new();
        state.tier0();

        state.run("123"); // push sentinel value on stack
        state.run(": the-answer 42 ;"); // define new word
        assert_eq!(state.pop_i32(), Some(123)); // make sure the word definition has no effect on the stack
        state.run("the-answer"); // run the new word
        assert_eq!(state.pop_i32(), Some(42));
    }

    #[test]
    fn new_parse_words() {
        let mut state = State::new();
        state.tier0();

        state.add_native_word("-rot", "(a b c -- c a b)",|state| {
            let a = state.pop();
            let b = state.pop();
            let c = state.pop();
            state.push(a);
            state.push(c);
            state.push(b);
        });
        state.add_native_word(".s", "( -- )", |state| println!("{:?}", state.stack));

        state.run("123"); // push sentinel value on stack
        state.run("SYNTAX: the-answer 42 -rot ;"); // define new parse word that puts a number deep in the stack
        state.run(": nop the-answer ; .s"); // define a new word
        assert_eq!(state.pop_i32(), Some(42)); // the number should end up on the stack during word definition
        state.run("nop"); // make sure the new word does nothing
        assert_eq!(state.pop_i32(), Some(123));
    }
}
