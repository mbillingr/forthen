use std::collections::HashMap;
use std::rc::Rc;

use crate::object::Object;
use crate::rcstring::RcString;

#[derive(Debug)]
pub enum Word {
    Word(Object),
    ParsingWord(Object),
}

#[derive(Debug)]
pub struct Entry {
    pub name: Rc<String>,
    pub word: Word,
}

#[derive(Debug, Copy, Clone)]
pub struct WordId(usize);

#[derive(Debug)]
pub struct Dictionary {
    entries: Vec<Entry>,
    words: HashMap<RcString, WordId>,
}

impl Dictionary {
    pub fn new() -> Self {
        Dictionary {
            entries: Vec::new(),
            words: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: Rc<String>, val: Entry) {
        let id = WordId(self.entries.len());
        self.entries.push(val);
        self.words.insert(key.into(), id);
    }

    pub fn get(&self, id: WordId) -> &Entry {
        &self.entries[id.0]
    }

    pub fn lookup(&self, key: &str) -> Option<WordId> {
        self.words.get(key).cloned()
    }
}
