use std::collections::HashSet;
use std::rc::Rc;

use crate::object::Object;
use crate::rcstring::RcString;

/// will be responsible for things like string and small integer reuse
#[derive(Debug)]
pub struct ObjectFactory {
    strings: HashSet<RcString>,
}

impl ObjectFactory {
    pub fn new() -> Self {
        ObjectFactory {
            strings: HashSet::new(),
        }
    }

    pub fn parse(&mut self, s: &str) -> Option<Object> {
        if s.starts_with('"') && s.ends_with('"') {
            Some(self.get_string(&s[1..s.len() - 1]).into())
        } else {
            s.parse::<i32>().ok().map(Object::from)
        }
    }

    pub fn get_string(&mut self, s: &str) -> Rc<String> {
        if let Some(rcs) = self.strings.get(s) {
            rcs.clone().into()
        } else {
            let rcs = Rc::new(s.to_owned());
            self.strings.insert(rcs.clone().into());
            rcs
        }
    }

    pub fn new_list(&self) -> Object {
        Object::List(Rc::new(vec![]))
    }
}
