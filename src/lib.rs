use std::cell::{RefCell, RefMut, Ref};
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;


#[derive(Clone)]
enum Callable {
    Native(fn(&mut State)),

}

impl Callable {
    fn invoke(&self, state: &mut State) {
        match self {
            Callable::Native(fun) => fun(state),
        }
    }
}


#[derive(Clone)]
enum Object {
    None,
    Callable(Callable),
    List(Rc<RefCell<Vec<Object>>>),
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
            other => state.push(other)
        }
    }

    fn as_vec_mut(&mut self) -> RefMut<Vec<Object>> {
        match self {
            Object::List(vec) => vec.borrow_mut(),
            _ => panic!("Type Error"),
        }
    }

    fn as_vec(&self) -> Ref<Vec<Object>> {
        match self {
            Object::List(vec) => vec.borrow(),
            _ => panic!("Type Error"),
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
            Some(self.get_string(&s[1..s.len()-1]).into())
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
        Object::List(Rc::new(RefCell::new(vec![])))
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
        self.push(self.factory.new_list());
        while let Some(token) = self.input_tokens.pop_front() {
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
        self.run_sequence(ops.as_vec().as_slice());
    }

    fn run_sequence(&mut self, ops: &[Object]) {
        for op in ops {
            op.clone().invoke(self);
        }
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

    pub fn tier0(&mut self) {
        self.dictionary.insert(self.factory.get_string(".s"), Entry::Word(Object::Callable(Callable::Native(|state| println!("{:?}", state.stack)))));
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
    fn it_works() {
        let mut state = State::new();
        state.tier0();

        state.run("3 5 \"hello forth!\" .s");
        state.run("3 5 \"hello forth!\" .s");

        println!("{:#?}", state);

        panic!("panicking so we can see the output :)");
    }
}
