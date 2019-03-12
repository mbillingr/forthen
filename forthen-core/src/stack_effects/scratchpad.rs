use super::element::{Element, ElementRef};

#[derive(Debug, Default)]
pub struct Scratchpad {
    elements: Vec<ElementRef>,
}

impl Scratchpad {
    pub fn find_or_insert(&mut self, new_node: Element) -> ElementRef {
        for noderef in &self.elements {
            if noderef.borrow().name() == new_node.name(){
                // todo: handle specialization and substitution
                //       e.g.  f( ... -- ... ) <-> f, and whatever

                noderef.borrow_mut().replace_if_more_specific(new_node).unwrap();

                return noderef.clone();
            }
        }

        self.insert(new_node)
    }

    pub fn insert(&mut self, new_node: Element) -> ElementRef {
        let r = ElementRef::new(new_node);
        self.elements.push(r.clone());
        r
    }

    pub fn find_by_name(&self, name: &str) -> Option<&ElementRef> {
        self.elements.iter().find(|e| e.borrow().name() == Some(name))
    }
}