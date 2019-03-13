use crate::dictionary::{Dictionary, Entry, WordId};
use crate::rcstring::RcString;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

#[derive(Debug, Default, Clone)]
pub struct ModuleRef(Rc<RefCell<Module>>);

impl ModuleRef {
    pub fn new() -> ModuleRef {
        Default::default()
    }

    /// Create a new module and add it to submodules under the given name.
    pub fn new_submodule(&self, name: String) -> ModuleRef {
        let newmod = Module {
            parent: Rc::downgrade(&self.0),
            submodules: HashMap::new(),
            dict: Dictionary::new(),
        };

        let rcmod = ModuleRef(Rc::new(RefCell::new(newmod)));

        self.0.borrow_mut().submodules.insert(name, rcmod.clone());

        rcmod
    }
    pub fn insert(&self, key: Rc<String>, val: Entry) {
        self.0.borrow_mut().insert(key, val)
    }

    pub fn lookup(&self, key: &str) -> Option<WordId> {
        self.0.borrow().lookup(key)
    }

    pub fn keys(&self) -> Vec<RcString> {
        self.0.borrow().keys()
    }

    pub fn local_lookup(&self, key: &str) -> Option<WordId> {
        self.0.borrow().local_lookup(key).cloned()
    }

    pub fn local_keys(&self) -> Vec<RcString> {
        self.0.borrow().local_keys()
    }
}

#[derive(Debug, Default)]
pub struct Module {
    parent: Weak<RefCell<Module>>,
    submodules: HashMap<String, ModuleRef>,
    dict: Dictionary,
}

impl Module {
    pub fn insert(&mut self, key: Rc<String>, val: Entry) {
        self.dict.insert(key, val)
    }

    pub fn lookup(&self, key: &str) -> Option<WordId> {
        if let Some(word) = self.dict.lookup(key) {
            Some(word.clone())
        } else if let Some(parent) = self.parent.upgrade() {
            parent.borrow().lookup(key)
        } else {
            None
        }
    }

    pub fn keys(&self) -> Vec<RcString> {
        let mut keys = if let Some(parent) = self.parent.upgrade() {
            parent.borrow().keys()
        } else {
            vec![]
        };
        keys.extend(self.dict.keys());
        keys
    }

    pub fn local_lookup(&self, key: &str) -> Option<&WordId> {
        self.dict.lookup(key)
    }

    pub fn local_keys(&self) -> Vec<RcString> {
        self.dict.keys()
    }
}
