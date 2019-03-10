use std::collections::HashSet;
use std::rc::Rc;

use crate::objects::Object;
use crate::rcstring::RcString;

/// will be responsible for things like string and small integer reuse
#[derive(Debug, Default)]
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

    pub fn new_list(&self) -> Object {
        Object::List(Rc::new(vec![]))
    }
}

pub trait StringManager<S> {
    fn get_string(&mut self, s: S) -> Rc<String>;
}

impl StringManager<&str> for ObjectFactory {
    fn get_string(&mut self, s: &str) -> Rc<String> {
        if let Some(rcs) = self.strings.get(s) {
            rcs.clone().into()
        } else {
            let rcs = Rc::new(s.to_owned());
            self.strings.insert(rcs.clone().into());
            rcs
        }
    }
}

impl StringManager<String> for ObjectFactory {
    fn get_string(&mut self, s: String) -> Rc<String> {
        if let Some(rcs) = self.strings.get(s.as_str()) {
            rcs.clone().into()
        } else {
            let rcs = Rc::new(s);
            self.strings.insert(rcs.clone().into());
            rcs
        }
    }
}

impl StringManager<Rc<String>> for ObjectFactory {
    fn get_string(&mut self, rcs: Rc<String>) -> Rc<String> {
        // todo: do we want to make sure rcs is inserted into self.strings?
        rcs
    }
}

impl StringManager<Object> for ObjectFactory {
    fn get_string(&mut self, obj: Object) -> Rc<String> {
        obj.into()
    }
}
