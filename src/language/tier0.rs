use crate::dictionary::Entry;
use crate::object::Object;
use crate::state::State;

impl State {
    /// Load language tier 0 into the dictionary
    ///
    /// Tier 0 contains low level native words required for extending the language
    pub fn tier0(&mut self) {
        self.add_native_parse_word(":", |state| {
            let name = state.next_token().expect("word name");
            state.push_str(&name);
            state.begin_compile();
        });

        self.add_native_parse_word(";", |state| {
            let ops = state.pop();
            let name = state.pop();
            state.add_compound_word(name, ops.into_rc_vec());
        });
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
}
