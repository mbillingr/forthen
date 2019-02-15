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

    /// Load language tier 1 into the dictionary
    ///
    /// Tier 1 contains low level native words that form the basic building blocks of the language
    pub fn tier1(&mut self) {
        self.add_native_word(".s", |state| println!("{:?}", state.stack));
    }
}
