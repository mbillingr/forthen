use std::cell::{Ref, RefCell, RefMut};
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;

#[derive(Clone)]
enum Callable {
    Native(fn(&mut State)),
    Compound(Rc<Vec<Object>>),
}

impl Callable {
    fn invoke(&self, state: &mut State) {
        match self {
            Callable::Native(fun) => fun(state),
            Callable::Compound(ops) => state.run_sequence(&ops[..]),
        }
    }
}

#[derive(Clone)]
enum Object {
    None,
    Callable(Callable),
    List(Rc<Vec<Object>>),
    String(Rc<String>),
    I32(i32),
}

impl std::fmt::Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Object::None => write!(f, "None"),
            Object::Callable(_) => write!(f, "<func>"),
            Object::List(list) => write!(f, "{:?}", list),
            Object::String(rcs) => write!(f, "{:?}", rcs),
            Object::I32(i) => write!(f, "{:?}", i),
        }
    }
}

impl From<Rc<String>> for Object {
    fn from(s: Rc<String>) -> Object {
        Object::String(s)
    }
}

impl From<i32> for Object {
    fn from(i: i32) -> Object {
        Object::I32(i)
    }
}

impl Object {
    fn invoke(self, state: &mut State) {
        match self {
            Object::Callable(fun) => fun.invoke(state),
            other => state.push(other),
        }
    }

    fn as_vec_mut(&mut self) -> &mut Vec<Object> {
        match self {
            Object::List(vec) => Rc::get_mut(vec).expect("Unable to mutate list"),
            _ => panic!("Type Error"),
        }
    }

    fn as_slice(&self) -> &[Object] {
        match self {
            Object::List(vec) => &vec,
            _ => panic!("Type Error"),
        }
    }

    fn into_rc_vec(self) -> Rc<Vec<Object>> {
        match self {
            Object::List(vec) => vec,
            _ => panic!("Type Error"),
        }
    }

    fn into_rc_string(self) -> Rc<String> {
        match self {
            Object::String(rcs) => rcs,
            _ => panic!("Type Error"),
        }
    }

    fn try_into_i32(self) -> Option<i32> {
        match self {
            Object::I32(i) => Some(i),
            _ => None,
        }
    }
}

/// Wrap `Rc<String>` so that we can implement `Borrow<str>` on it
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct RcString(Rc<String>);

impl std::borrow::Borrow<str> for RcString {
    fn borrow(&self) -> &str {
        &self.0[..]
    }
}

/// will be responsible for things like string and small integer reuse
#[derive(Debug)]
struct ObjectFactory {
    strings: HashSet<RcString>,
}

impl ObjectFactory {
    fn new() -> Self {
        ObjectFactory {
            strings: HashSet::new(),
        }
    }

    fn parse(&mut self, s: &str) -> Option<Object> {
        if s.starts_with('"') && s.ends_with('"') {
            Some(self.get_string(&s[1..s.len() - 1]).into())
        } else {
            s.parse::<i32>().ok().map(Object::from)
        }
    }

    fn get_string(&mut self, s: &str) -> Rc<String> {
        if let Some(rcs) = self.strings.get(s) {
            rcs.0.clone().into()
        } else {
            let rcs = RcString(Rc::new(s.to_owned()));
            self.strings.insert(rcs.clone());
            rcs.0.into()
        }
    }

    fn new_list(&self) -> Object {
        Object::List(Rc::new(vec![]))
    }
}

#[derive(Debug)]
enum Entry {
    Word(Object),
    ParsingWord(Object),
}

#[derive(Debug)]
struct Dictionary {
    words: HashMap<RcString, Entry>,
}

impl Dictionary {
    fn new() -> Self {
        Dictionary {
            words: HashMap::new(),
        }
    }

    fn insert(&mut self, key: Rc<String>, val: Entry) {
        self.words.insert(RcString(key), val);
    }

    fn lookup(&self, key: &str) -> Option<&Entry> {
        self.words.get(key)
    }
}

#[derive(Debug)]
pub struct State {
    input_tokens: VecDeque<String>,
    stack: Vec<Object>,
    dictionary: Dictionary,
    factory: ObjectFactory,
}

impl State {
    pub fn new() -> Self {
        State {
            input_tokens: VecDeque::new(),
            stack: vec![],
            dictionary: Dictionary::new(),
            factory: ObjectFactory::new(),
        }
    }

    pub fn run(&mut self, input: &str) {
        self.input_tokens
            .extend(tokenize(input).map(str::to_string));
        self.begin_compile();
        while let Some(token) = self.next_token() {
            let literal = self.factory.parse(&token);
            let word = self.dictionary.lookup(&token);
            match (literal, word) {
                (None, None) => panic!("Unknown Word: {}", token),
                (Some(_), Some(_)) => panic!("Ambiguous Word: {}", token),
                (Some(obj), None) => self.top_mut().as_vec_mut().push(obj),
                (None, Some(Entry::Word(obj))) => {
                    let obj = obj.clone();
                    self.top_mut().as_vec_mut().push(obj);
                }
                (None, Some(Entry::ParsingWord(obj))) => obj.clone().invoke(self),
            }
        }
        let ops = self.pop();
        self.run_sequence(ops.as_slice());
    }

    fn run_sequence(&mut self, ops: &[Object]) {
        for op in ops {
            op.clone().invoke(self);
        }
    }

    fn next_token(&mut self) -> Option<String> {
        self.input_tokens.pop_front()
    }

    fn push(&mut self, obj: Object) {
        self.stack.push(obj);
    }

    fn pop(&mut self) -> Object {
        self.stack.pop().expect("Stack Empty")
    }

    fn top_mut(&mut self) -> &mut Object {
        self.stack.last_mut().expect("Stack Empty")
    }

    fn push_str(&mut self, s: &str) {
        let obj = self.factory.get_string(s).into();
        self.push(obj);
    }

    fn pop_i32(&mut self) -> Option<i32> {
        self.pop().try_into_i32()
    }

    fn pop_str(&mut self) -> Option<String> {
        let obj = self.pop();
        let rcs = obj.into_rc_string();
        match Rc::try_unwrap(rcs) {
            Ok(s) => Some(s),
            Err(rcs) => Some((*rcs).clone()),
        }
    }

    fn begin_compile(&mut self) {
        self.push(self.factory.new_list());
    }

    fn swap(&mut self) {
        let a = self.pop();
        let b = self.pop();
        self.push(a);
        self.push(b);
    }

    pub fn tier0(&mut self) {
        self.dictionary.insert(
            self.factory.get_string(".s"),
            Entry::Word(Object::Callable(Callable::Native(|state| {
                println!("{:?}", state.stack)
            }))),
        );

        self.dictionary.insert(
            self.factory.get_string(":"),
            Entry::ParsingWord(Object::Callable(Callable::Native(|state| {
                let name = state.next_token().expect("word name");
                state.push_str(&name);
                state.begin_compile();
            }))),
        );

        self.dictionary.insert(
            self.factory.get_string(";"),
            Entry::ParsingWord(Object::Callable(Callable::Native(|state| {
                let ops = state.pop();
                let name = state.pop();

                state.dictionary.insert(
                    name.into_rc_string(),
                    Entry::Word(Object::Callable(Callable::Compound(ops.into_rc_vec()))),
                )
            }))),
        );
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

fn skip_while(
    it: &mut std::iter::Peekable<std::str::CharIndices>,
    predicate: impl Fn(char) -> bool,
) {
    while let Some(true) = it.peek().map(|&(_, ch)| predicate(ch)) {
        it.next();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn it_works() {
        let mut state = State::new();
        state.tier0();

        state.run("3 5 \"hello forth!\" .s");
        state.run("3 5 \"hello forth!\" .s");

        state.run(": the-answer 42 ;");
        state.run("the-answer .s");

        println!("{:#?}", state);

        panic!("panicking so we can see the output :)");
    }
}
