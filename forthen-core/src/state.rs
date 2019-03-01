use std::borrow::Borrow;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::dictionary::{Dictionary, Entry, Word};
use crate::object::Object;
use crate::object_factory::{ObjectFactory, StringManager};
use crate::parsing::tokenize;
use crate::scope::CompilerScope;
use crate::stack_effect::{IntoStackEffect, StackEffect};
use crate::vm::{Opcode, Quotation};

#[derive(Debug)]
pub struct State {
    input_tokens: VecDeque<String>,
    pub stack: Vec<Object>,
    pub frames: Vec<Object>,
    pub dictionary: Dictionary,
    pub factory: ObjectFactory,
    pub scopes: Vec<CompilerScope>,
}

/// API
impl State {
    pub fn new() -> Self {
        State {
            input_tokens: VecDeque::new(),
            stack: vec![],
            frames: vec![],
            dictionary: Dictionary::new(),
            factory: ObjectFactory::new(),
            scopes: vec![],
        }
    }

    pub fn run(&mut self, input: &str) {
        self.input_tokens
            .extend(tokenize(input).map(str::to_string));
        self.begin_compile();
        while let Some(token) = self.next_token() {
            self.parse_token(&token);
        }
        let quot = self.pop().into_rc_quotation();
        quot.run(self)
    }

    pub fn run_sequence(&mut self, ops: &[Object]) {
        for op in ops {
            op.clone().invoke(self);
        }
    }

    pub fn next_token(&mut self) -> Option<String> {
        self.input_tokens.pop_front()
    }

    pub fn parse_until(&mut self, delimiter: &str) {
        loop {
            match self.next_token() {
                None => panic!("Parse Error"),
                Some(ref token) if token == delimiter => break,
                Some(token) => self.parse_token(&token),
            }
        }
    }

    pub fn parse_token(&mut self, token: &str) {
        // todo: i don't know yet which takes up more time - parsing or lookup...
        //       so we always do them both now, and future profiling will show which to do first in the future
        let literal = self.factory.parse(&token);
        let word = self.dictionary.lookup(&token);
        match (literal, word) {
            (None, None) => panic!("Unknown Word: {}", token),
            (Some(_), Some(_)) => panic!("Ambiguous Word: {}", token),
            (Some(obj), None) => self
                .top_mut()
                .as_quotation_mut()
                .ops
                .push(Opcode::Push(obj)),
            (None, Some(entry)) => match &entry.word {
                Word::Word(_) => {
                    let op = Opcode::call_word(entry.clone());
                    self.top_mut().as_quotation_mut().ops.push(op);
                }
                Word::ParsingWord(obj) => obj.clone().invoke(self),
            },
        }
    }

    pub fn add_native_word<S>(
        &mut self,
        name: S,
        stack_effect: impl IntoStackEffect,
        func: fn(&mut State),
    ) where
        ObjectFactory: StringManager<S>,
    {
        let name = self.factory.get_string(name);
        self.dictionary.insert(
            name.clone(),
            Entry {
                name,
                word: Word::Word(Object::NativeFunction(
                    func,
                    stack_effect.into_stack_effect(),
                )),
            },
        );
    }

    pub fn add_native_parse_word<S>(&mut self, name: S, func: fn(&mut State))
    where
        ObjectFactory: StringManager<S>,
    {
        let name = self.factory.get_string(name);
        self.dictionary.insert(
            name.clone(),
            Entry {
                name,
                word: Word::ParsingWord(Object::NativeFunction(func, StackEffect::new_mod("acc"))),
            },
        );
    }

    pub fn add_closure_parse_word<S>(&mut self, name: S, func: impl Fn(&mut State) + 'static)
    where
        ObjectFactory: StringManager<S>,
    {
        let name = self.factory.get_string(name);
        self.dictionary.insert(
            name.clone(),
            Entry {
                name,
                word: Word::ParsingWord(Object::NativeClosure(
                    Rc::new(func),
                    StackEffect::new_mod("acc"),
                )),
            },
        );
    }

    pub fn add_compound_word<S>(
        &mut self,
        name: S,
        stack_effect: impl IntoStackEffect,
        quot: Rc<Quotation>,
    ) where
        ObjectFactory: StringManager<S>,
    {
        let name = self.factory.get_string(name);
        self.dictionary.insert(
            name.clone(),
            Entry {
                name,
                word: Word::Word(Object::Quotation(quot, stack_effect.into_stack_effect())),
            },
        );
    }

    pub fn add_compound_parse_word<S>(&mut self, name: S, quot: Rc<Quotation>)
    where
        ObjectFactory: StringManager<S>,
    {
        let name = self.factory.get_string(name);
        self.dictionary.insert(
            name.clone(),
            Entry {
                name,
                word: Word::ParsingWord(Object::Quotation(quot, StackEffect::new_mod("acc"))),
            },
        );
    }

    pub fn format_word(&self, name: &str) {
        let entry = self.dictionary.lookup(name);
        match entry {
            None => println!("{:>20}  undefined!", name),
            Some(entry) => match entry.word.inner() {
                Object::NativeFunction(_, se) => {
                    println!("{:>20}   {:50}   <native>", entry.name, format!("({})", se))
                }
                Object::Quotation(quot, se) => {
                    let ops: Vec<_> = quot
                        .ops
                        .iter()
                        .map(|op| match op {
                            Opcode::Call(Object::Word(entry)) => format!("{}", entry.name),
                            Opcode::Call(obj) | Opcode::Push(obj) => format!("{:?}", obj),
                            Opcode::TailRecurse => format!("tail-recurse"),
                        })
                        .collect();
                    println!(
                        "{:>20}   {:50}   {}",
                        entry.name,
                        format!("({})", se),
                        ops.join(" ")
                    )
                }
                _ => println!("{:>20}  invalid word", name),
            },
        }
    }

    pub fn print_dictionary(&self) {
        let mut words = self.dictionary.keys();
        words.sort();
        for word in words {
            self.format_word(word.borrow());
        }
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

    pub fn pop_bool(&mut self) -> Option<bool> {
        self.pop().try_into_bool()
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
        self.push(Object::Quotation(
            Rc::new(Quotation::new()),
            StackEffect::new(),
        ));
    }

    pub fn dup(&mut self) {
        let a = self.pop();
        self.push(a.clone());
        self.push(a);
    }

    pub fn swap(&mut self) {
        let a = self.pop();
        let b = self.pop();
        self.push(a);
        self.push(b);
    }

    pub fn over(&mut self) {
        let b = self.pop();
        let a = self.pop();
        self.push(a.clone());
        self.push(b);
        self.push(a);
    }
}
