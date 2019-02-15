use std::rc::Rc;
use std::collections::{HashMap, VecDeque};

enum Object {
    None,
    Native(fn(&mut State)),
}


enum Entry {
    Object(Rc<Object>)
}


struct Dictionary {
    words: HashMap<String, Entry>
}

impl Dictionary {
    fn new() -> Self {
        Dictionary {
            words: HashMap::new()
        }
    }

    fn insert(&mut self, key: String, val: Entry) {
        self.words.insert(key, val);
    }
}


struct State {
    input_tokens: VecDeque<String>,
    stack: Vec<Rc<Object>>,
    dictionary: Dictionary,
}

impl State {
    pub fn new() -> Self {
        State {
            input_tokens: VecDeque::new(),
            stack: vec![],
            dictionary: Dictionary::new(),
        }
    }

    pub fn run(&mut self, input: &str) {
        self.input_tokens.extend(tokenize(input).map(str::to_string));
        while let Some(token) = self.input_tokens.pop_front() {
            unimplemented!()
        }
    }

    pub fn tier0(&mut self) {
        unimplemented!()
    }
}


fn tokenize(input: &str) -> impl Iterator<Item = &str> {
    let mut it = input.char_indices().peekable();

    std::iter::repeat(())
        .map(move |_| {
            skip_while(&mut it, char::is_whitespace);

            match it.peek() {
                None => return None,
                Some((_, '"')) => {
                    let (a, _) = it.next().unwrap();
                    skip_while(&mut it, |ch| ch != '"');
                    it.next();
                    match it.peek() {
                        Some((b, _)) => return Some(&input[a..*b]),
                        None => return Some(&input[a..]),
                    }
                }
                Some((i, _)) => {
                    let a = *i;
                    skip_while(&mut it, |ch| !ch.is_whitespace());
                    match it.peek() {
                        Some((b, _)) => return Some(&input[a..*b]),
                        None => return Some(&input[a..]),
                    }
                }
            }
        })
        .take_while(Option::is_some)
        .map(Option::unwrap)
}

fn skip_while(it: &mut std::iter::Peekable<std::str::CharIndices>, predicate: impl Fn(char) -> bool) {
    while let Some(true) = it.peek().map(|&(_, ch)| predicate(ch)) {
        it.next();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut state = State::new();
        state.run("3 5 +");
    }
}
