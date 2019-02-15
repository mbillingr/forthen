use std::collections::VecDeque;
use std::rc::Rc;

use crate::dictionary::{Dictionary, Entry};
use crate::object::Object;
use crate::object_factory::{ObjectFactory, StringManager};
use crate::parsing::tokenize;

#[derive(Debug)]
pub struct State {
    input_tokens: VecDeque<String>,
    pub stack: Vec<Object>,
    pub dictionary: Dictionary,
    pub factory: ObjectFactory,
}

/// API
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

    pub fn run_sequence(&mut self, ops: &[Object]) {
        for op in ops {
            op.clone().invoke(self);
        }
    }

    pub fn next_token(&mut self) -> Option<String> {
        self.input_tokens.pop_front()
    }

    pub fn add_native_word<S>(&mut self, name: S, func: fn(&mut State))
    where
        ObjectFactory: StringManager<S>,
    {
        self.dictionary.insert(
            self.factory.get_string(name),
            Entry::Word(Object::NativeFunction(func)),
        );
    }

    pub fn add_native_parse_word<S>(&mut self, name: S, func: fn(&mut State))
    where
        ObjectFactory: StringManager<S>,
    {
        self.dictionary.insert(
            self.factory.get_string(name),
            Entry::ParsingWord(Object::NativeFunction(func)),
        );
    }

    pub fn add_compound_word<S>(&mut self, name: S, ops: Rc<Vec<Object>>)
    where
        ObjectFactory: StringManager<S>,
    {
        self.dictionary.insert(
            self.factory.get_string(name),
            Entry::Word(Object::CompoundFunction(ops)),
        );
    }

    pub fn clear_stack(&mut self) {
        self.stack.clear();
    }

    pub fn push(&mut self, obj: Object) {
        self.stack.push(obj);
    }

    pub fn pop(&mut self) -> Object {
        self.stack.pop().expect("Stack Empty")
    }

    pub fn top_mut(&mut self) -> &mut Object {
        self.stack.last_mut().expect("Stack Empty")
    }

    pub fn push_str(&mut self, s: &str) {
        let obj = self.factory.get_string(s).into();
        self.push(obj);
    }

    pub fn pop_i32(&mut self) -> Option<i32> {
        self.pop().try_into_i32()
    }

    pub fn pop_str(&mut self) -> Option<String> {
        let obj = self.pop();
        let rcs = obj.into();
        match Rc::try_unwrap(rcs) {
            Ok(s) => Some(s),
            Err(rcs) => Some((*rcs).clone()),
        }
    }

    pub fn begin_compile(&mut self) {
        self.push(self.factory.new_list());
    }

    pub fn swap(&mut self) {
        let a = self.pop();
        let b = self.pop();
        self.push(a);
        self.push(b);
    }
}
