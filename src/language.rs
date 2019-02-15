use crate::dictionary::Entry;
use crate::object::Object;
use crate::state::State;

impl State {
    pub fn tier0(&mut self) {
        self.dictionary.insert(
            self.factory.get_string(".s"),
            Entry::Word(Object::NativeFunction(|state| {
                println!("{:?}", state.stack)
            })),
        );

        self.dictionary.insert(
            self.factory.get_string(":"),
            Entry::ParsingWord(Object::NativeFunction(|state| {
                let name = state.next_token().expect("word name");
                state.push_str(&name);
                state.begin_compile();
            })),
        );

        self.dictionary.insert(
            self.factory.get_string(";"),
            Entry::ParsingWord(Object::NativeFunction(|state| {
                let ops = state.pop();
                let name = state.pop();

                state.dictionary.insert(
                    name.into_rc_string(),
                    Entry::Word(Object::CompoundFunction(ops.into_rc_vec())),
                )
            })),
        );
    }
}
