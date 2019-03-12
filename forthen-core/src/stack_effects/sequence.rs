
use super::element::{Element, ElementRef};
use super::effect::StackEffect;

pub fn normalized_sequence(seq: Vec<ElementRef>) -> Vec<ElementRef> {
    let mut out = vec![];

    for el in seq {
        if let Some(mut borrow) = el.try_borrow_mut() {
            match *borrow {
                Element::Ellipsis(_) => out.push(el.clone()),
                Element::Item(_) => out.push(el.clone()),
                Element::Callable(_, ref mut se) => {
                    *se = se.clone().normalized();
                    out.push(el.clone());
                },
                Element::Sequence(ref s) => {
                    out.extend(normalized_sequence(s.clone()));
                },
            }
        } else {
            // looks like we are already working on this ((recursive?) callable?)... 
            out.push(el.clone());
        }
    }

    out
}
