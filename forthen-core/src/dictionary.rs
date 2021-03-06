use std::collections::HashMap;
use std::rc::Rc;

use crate::objects::Object;
use crate::rcstring::RcString;
use crate::vm::ByteCode;
use crate::StackEffect;

#[derive(Debug)]
pub enum Word {
    Word(Object),
    ParsingWord(Object),
}

impl Word {
    pub fn inner(&self) -> &Object {
        match self {
            Word::Word(obj) | Word::ParsingWord(obj) => obj,
        }
    }
}

#[derive(Debug)]
pub struct Entry {
    pub name: Rc<String>,
    pub word: Word,
    pub source: Option<Rc<ByteCode>>,
    pub stack_effect: StackEffect,
}

pub type WordId = Rc<Entry>;

#[derive(Debug, Default)]
pub struct Dictionary {
    words: HashMap<RcString, Rc<Entry>>,
}

impl Dictionary {
    pub fn new() -> Self {
        Dictionary {
            words: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: Rc<String>, val: Entry) {
        self.words.insert(key.into(), Rc::new(val));
    }

    pub fn insert_ref(&mut self, key: Rc<String>, val: WordId) {
        self.words.insert(key.into(), val);
    }

    pub fn lookup(&self, key: &str) -> Option<&WordId> {
        self.words.get(key)
    }

    pub fn keys(&self) -> Vec<RcString> {
        self.words.keys().cloned().collect()
    }
}

impl std::fmt::Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
