use std::borrow::Borrow;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::dictionary::{Entry, Word, WordId};
use crate::errors::*;
use crate::module::ModuleRef;
use crate::object_factory::{ObjectFactory, StringManager};
use crate::objects::{callable::Callable, prelude::*};
use crate::parsing::tokenize;
use crate::scope::CompilerScope;
use crate::stack_effects::{IntoStackEffect, StackEffect};

#[derive(Debug, Copy, Clone)]
pub enum Mode {
    Eval,
    Compile,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Eval
    }
}

#[derive(Debug, Default)]
pub struct State {
    input_tokens: VecDeque<String>,
    pub stack: Vec<Object>,
    pub frames: Vec<Object>,
    pub factory: ObjectFactory,
    pub scopes: Vec<CompilerScope>,
    pub current_module: ModuleRef,
    root_module: ModuleRef,
    mode: Vec<Mode>,
}

/// API
impl State {
    pub fn new() -> Self {
        let root_module = ModuleRef::new();
        State {
            input_tokens: VecDeque::new(),
            stack: vec![],
            frames: vec![],
            current_module: root_module.clone(),
            factory: ObjectFactory::new(),
            mode: vec![],
            scopes: vec![],
            root_module,
        }
    }

    /// create new state that shares modules with the current state
    pub fn substate(&self) -> Self {
        State {
            input_tokens: VecDeque::new(),
            stack: vec![],
            frames: vec![],
            current_module: self.current_module.clone(),
            factory: ObjectFactory::new(),
            mode: vec![],
            scopes: vec![],
            root_module: self.root_module.clone(),
        }
    }

    pub fn current_mode(&self) -> Mode {
        self.mode.last().cloned().unwrap_or_else(|| Mode::default())
    }

    pub fn run(&mut self, input: &str) -> Result<()> {
        self.input_tokens
            .extend(tokenize(input).map(str::to_string));

        while let Some(token) = self.next_token() {
            match self.parse_token(&token) {
                Ok(_) => {}
                err @ Err(_) => {
                    self.input_tokens.clear();
                    return err
                }
            }
        }

        Ok(())
    }

    pub fn next_token(&mut self) -> Option<String> {
        self.input_tokens.pop_front()
    }

    pub fn parse_until(&mut self, delimiter: &str) -> Result<()> {
        loop {
            match self.next_token() {
                None => return Err(ErrorKind::EndOfInput.into()),
                Some(ref token) if token == delimiter => break,
                Some(token) => self.parse_token(&token)?,
            }
        }
        Ok(())
    }

    pub fn parse_token(&mut self, token: &str) -> Result<()> {
        // todo: i don't know yet which takes up more time - parsing or lookup...
        //       so we always do them both now, and future profiling will show which to do first in the future
        let literal = self.factory.parse(&token);
        let word = self.current_module.lookup(&token);
        let mode = self.mode.last().unwrap_or(&Mode::Eval);
        match (mode, literal, word) {
            (_, None, None) => return Err(ErrorKind::UnknownWord(token.to_string()).into()),
            (_, Some(_), Some(_)) => return Err(ErrorKind::AmbiguousWord(token.to_string()).into()),
            (Mode::Eval, Some(obj), None) => self.push(obj)?,
            (Mode::Compile, Some(obj), None) => self.top_mut()?.as_vec_mut()?.push(obj),
            (Mode::Eval, None, Some(entry)) => entry.word.inner().call(self)?,
            (Mode::Compile, None, Some(entry)) => match &entry.word {
                Word::Word(_) => {
                    //let op = Opcode::call_word(entry.clone());
                    let op = Object::Word(entry);
                    self.top_mut()?.as_vec_mut()?.push(op);
                }
                Word::ParsingWord(obj) => obj.clone().call(self)?,
            },
        }
        Ok(())
    }

    pub fn add_native_word<S>(
        &mut self,
        name: S,
        stack_effect: impl IntoStackEffect,
        func: impl Fn(&mut State) -> Result<()> + 'static,
    ) where
        ObjectFactory: StringManager<S>,
    {
        let name = self.factory.get_string(name);
        self.current_module.insert(
            name.clone(),
            Entry {
                name,
                word: Word::Word(Object::Function(Callable::new_const(func))),
                source: None,
                stack_effect: stack_effect.into_stack_effect(),
            },
        );
    }

    pub fn add_native_parse_word<S>(
        &mut self,
        name: S,
        func: impl Fn(&mut State) -> Result<()> + 'static,
    ) where
        ObjectFactory: StringManager<S>,
    {
        let name = self.factory.get_string(name);
        self.current_module.insert(
            name.clone(),
            Entry {
                name,
                word: Word::ParsingWord(Object::Function(Callable::new_const(func))),
                source: None,
                stack_effect: StackEffect::new_mod("acc"),
            },
        );
    }

    pub fn add_compound_word<S>(&mut self, name: S, stack_effect: impl IntoStackEffect, obj: Object)
    where
        ObjectFactory: StringManager<S>,
    {
        let name = self.factory.get_string(name);
        self.current_module.insert(
            name.clone(),
            Entry {
                name,
                source: None,
                word: Word::Word(obj),
                stack_effect: stack_effect.into_stack_effect(),
            },
        );
    }

    pub fn add_compound_parse_word<S>(&mut self, name: S, obj: Object)
    where
        ObjectFactory: StringManager<S>,
    {
        let name = self.factory.get_string(name);
        self.current_module.insert(
            name.clone(),
            Entry {
                name,
                source: None,
                word: Word::ParsingWord(obj),
                stack_effect: StackEffect::new_mod("acc"),
            },
        );
    }

    pub fn format_word(&self, name: &str) {
        let entry = self.current_module.lookup(name);
        match entry {
            None => println!("{:>20}  undefined!", name),
            Some(entry) => match entry.word.inner() {
                Object::Function(ca) => {
                    let func = match entry.source {
                        None => format!("<{:?}>", ca),
                        Some(ref byte_code) => format!("{}", byte_code),
                    };
                    println!(
                        "{:>20}   {:50}   {}",
                        entry.name,
                        format!("({})", entry.stack_effect),
                        func
                    )
                }
                Object::List(list) => {
                    println!(
                        "{:>20}   {:50}   {:?}",
                        entry.name,
                        format!("({})", entry.stack_effect),
                        list
                    );
                }
                _ => println!("{:>20}  invalid word", name),
            },
        }
    }

    pub fn print_dictionary(&self) {
        let mut words = self.current_module.keys();
        words.sort();
        for word in words {
            self.format_word(word.borrow());
        }
    }

    pub fn clear_stack(&mut self) {
        self.stack.clear();
    }

    pub fn push<T: Into<Object>>(&mut self, val: T) -> Result<()> {
        self.stack.push(val.into());
        Ok(())
    }

    pub fn pop(&mut self) -> Result<Object> {
        self.stack
            .pop()
            .ok_or_else(|| ErrorKind::StackUnderflow.into())
    }

    pub fn top(&mut self) -> Result<&Object> {
        self.stack
            .last()
            .ok_or_else(|| ErrorKind::StackUnderflow.into())
    }

    pub fn top_mut(&mut self) -> Result<&mut Object> {
        self.stack
            .last_mut()
            .ok_or_else(|| ErrorKind::StackUnderflow.into())
    }

    pub fn push_str(&mut self, s: &str) -> Result<()> {
        let obj = self.factory.get_string(s);
        self.push(obj)
    }

    pub fn push_string(&mut self, s: String) -> Result<()> {
        let obj = self.factory.get_string(s);
        self.push(obj)
    }

    pub fn pop_bool(&mut self) -> Result<bool> {
        self.pop()?.try_into_bool()
    }

    pub fn pop_i32(&mut self) -> Result<i32> {
        self.pop()?.try_into_i32()
    }

    pub fn pop_string(&mut self) -> Result<String> {
        let obj = self.pop()?;
        let rcs = obj.into();
        match Rc::try_unwrap(rcs) {
            Ok(s) => Ok(s),
            Err(rcs) => Ok((*rcs).clone()),
        }
    }

    pub fn compile<F: FnOnce(&mut Self) -> Result<()>>(&mut self, func: F) -> Result<()> {
        self.mode.push(Mode::Compile);
        self.push(Object::List(Rc::new(Vec::new()))).unwrap();
        match func(self) {
            Ok(_) => {
                self.mode.pop();
                Ok(())
            },
            err @ Err(_) => {
                self.pop().unwrap();
                self.mode.pop();
                err
            }
        }
    }

    pub fn compile_scoped<F: FnOnce(&mut Self) -> Result<()>>(&mut self, func: F) -> Result<CompilerScope> {
        self.scopes.push(CompilerScope::new());
        match self.compile(func) {
            Ok(_) => {
                self.scopes.pop().ok_or_else(||ErrorKind::Msg("Could not get scope".to_string()).into())
            },
            Err(e) => {
                self.scopes.pop();
                Err(e)
            }
        }
    }

    /*pub fn begin_compile(&mut self) {
        self.mode.push(Mode::Compile);
        self.push(Object::List(Rc::new(Vec::new()))).unwrap();
    }

    pub fn end_compile(&mut self) {
        self.mode.pop();
    }*/

    pub fn compile_object(&mut self, obj: Object) -> Result<()> {
        self.top_mut()?.as_vec_mut()?.push(obj);
        Ok(())
    }

    pub fn compile_word(&mut self, word: WordId) -> Result<()> {
        self.compile_object(Object::Word(word))
    }

    pub fn dup(&mut self) -> Result<()> {
        let a = self.pop()?;
        self.push(a.clone())?;
        self.push(a)
    }

    pub fn swap(&mut self) -> Result<()> {
        let a = self.pop()?;
        let b = self.pop()?;
        self.push(a)?;
        self.push(b)
    }

    pub fn over(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(a.clone())?;
        self.push(b)?;
        self.push(a)
    }

    pub fn rot(&mut self) -> Result<()> {
        let c = self.pop()?;
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(b)?;
        self.push(c)?;
        self.push(a)
    }

    pub fn root_module(&self) -> &ModuleRef {
        &self.root_module
    }

    pub fn new_mod(&mut self, name: String) -> Result<()> {
        let newmod = self.current_module.new_submodule(name);
        self.current_module = newmod;

        // We define the END-MODULE word only in new submodules.
        // This prevents accidentally ending the root module.
        // However, someone could still sneakily import this function
        // from another module and cause havoc in the root. For now,
        // we simply panic in this case. Ignoring or warning might
        // be fine too...
        self.add_native_parse_word("END-MODULE", |state| {
            state
                .exit_mod()
                .ok_or_else(|| panic!("Error: attempt to end root module"))
        });

        Ok(())
    }

    pub fn exit_mod(&mut self) -> Option<()> {
        self.current_module
            .parent()
            .map(|parent| self.current_module = parent)
    }
}
