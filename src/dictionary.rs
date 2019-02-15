use std::collections::HashMap;
use std::rc::Rc;

use crate::object::Object;
use crate::rcstring::RcString;

#[derive(Debug)]
pub enum Entry {
    Word(Object),
    ParsingWord(Object),
}

#[derive(Debug)]
pub struct Dictionary {
    words: HashMap<RcString, Entry>,
}

impl Dictionary {
    pub fn new() -> Self {
        Dictionary {
            words: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: Rc<String>, val: Entry) {
        self.words.insert(key.into(), val);
    }

    pub fn lookup(&self, key: &str) -> Option<&Entry> {
        self.words.get(key)
    }
}
