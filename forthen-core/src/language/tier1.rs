use crate::state::State;

impl State {
    /// Load language tier 1 into the dictionary
    ///
    /// Tier 1 contains low level native words that form the basic building blocks of the language
    pub fn tier1(&mut self) {
        // development tools
        self.add_native_word(".s", "( -- )", |state| println!("{:?}", state.stack));
        self.add_native_word(".", "( x -- )", |state| println!("{:?}", state.pop()));

        // stack operations
        self.add_native_word("dup", "(a -- a a)", State::dup);
        self.add_native_word("drop", "(a -- )", |state| {
            state.pop();
        });
        self.add_native_word("swap", "(a b -- b a)", State::swap);
        self.add_native_word("over", "(a b -- a b a)", State::over);
    }
}

#[cfg(test)]
mod tests {
    use crate::state::State;

    #[test]
    fn stack_ops() {
        let mut state = State::new();
        state.tier1();

        state.run("123 456 swap");
        state.assert_stack(&[456, 123]);

        state.clear_stack();

        state.run("\"abc\" \"def\" \"ghi\" swap");
        state.assert_stack(&["abc", "ghi", "def"]);

        state.clear_stack();

        state.run("123 \"abc\" swap");
        assert_eq!(state.pop(), 123);
        assert_eq!(state.pop(), "abc");
    }
}