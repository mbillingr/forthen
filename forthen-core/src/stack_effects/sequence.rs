use super::element::{Element, ElementHash, ElementRef};
use std::collections::HashMap;

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
                }
                Element::Sequence(ref s) => {
                    out.extend(normalized_sequence(s.clone()));
                }
            }
        } else {
            // looks like we are already working on this ((recursive?) callable?)...
            println!("***");
            out.push(el.clone());
        }
    }

    out
}

pub fn is_sequence_recursive_equivalent(
    a: &[ElementRef],
    b: &[ElementRef],
    mapping: &mut HashMap<usize, usize>,
) -> bool {
    use Element::*;

    if a.len() != b.len() {
        return false;
    }

    for (ea, eb) in a.iter().zip(b) {
        match mapping.get(&ea.addr()) {
            Some(aa) if aa == &eb.addr() => continue,
            Some(_) => return false,
            None => {}
        }

        match mapping.get(&eb.addr()) {
            Some(ab) if ab == &ea.addr() => continue,
            Some(_) => return false,
            None => {}
        }

        mapping.insert(ea.addr(), eb.addr());
        mapping.insert(eb.addr(), ea.addr());

        match (&*ea.borrow(), &*eb.borrow()) {
            (Sequence(_), _) | (_, Sequence(_)) => panic!(
                "Found nested sequence. Are you trying to compare unnormalized stack effects?"
            ),
            (Ellipsis(_), Ellipsis(_)) => {}
            (Item(_), Item(_)) => {}
            (Callable(_, sa), Callable(_, sb)) => {
                if !sa.is_recursive_equivalent(&sb, mapping) {
                    return false;
                }
            }
            _ => return false,
        }
    }

    true
}

pub fn sequence_recursive_deepcopy(
    seq: &[ElementRef],
    mapping: &mut HashMap<ElementHash, ElementRef>,
) -> Vec<ElementRef> {
    seq.iter()
        .map(|x| {
            let eh = x.clone().into();
            if let Some(y) = mapping.get(&eh) {
                y.clone()
            } else {
                let y = x.recursive_deepcopy(mapping);
                mapping.insert(eh, y.clone());
                y
            }
        })
        .collect()
}
