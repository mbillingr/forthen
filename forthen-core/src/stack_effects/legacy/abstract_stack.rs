use crate::errors::*;
use crate::refhash::RefHash;
use crate::stack_effect::{EffectNode, NodeRef, StackEffect};
use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::mem::replace;
use std::rc::Rc;

pub type ItemRef = RefHash<StackItem>;

pub enum StackItem {
    Row(String),
    Item(String),
    Quotation(String, RefCell<Sequence>, RefCell<Sequence>),
}

impl StackItem {
    pub fn anonymous_row() -> ItemRef {
        RefHash::new(Rc::new(StackItem::Row(String::new())))
    }

    pub fn item(name: &str) -> ItemRef {
        RefHash::new(Rc::new(StackItem::Item(name.to_string())))
    }

    pub fn row(name: &str) -> ItemRef {
        RefHash::new(Rc::new(StackItem::Row(name.to_string())))
    }

    pub fn quot(name: &str, inputs: &[ItemRef], outputs: &[ItemRef]) -> ItemRef {
        RefHash::new(Rc::new(StackItem::Quotation(
            name.to_string(),
            RefCell::new(inputs.iter().cloned().collect()),
            RefCell::new(outputs.iter().cloned().collect()),
        )))
    }

    pub fn quot_from_seqs(name: &str, inputs: Sequence, outputs: Sequence) -> ItemRef {
        RefHash::new(Rc::new(StackItem::Quotation(
            name.to_string(),
            RefCell::new(inputs),
            RefCell::new(outputs),
        )))
    }

    pub fn name(&self) -> &str {
        match self {
            StackItem::Row(name) | StackItem::Item(name) | StackItem::Quotation(name, _, _) => name,
        }
    }

    fn compare(&self, other: &Self) -> ItemOrd {
        use ItemOrd::*;
        use StackItem::*;

        match (self, other) {
            (Row(_), Row(_)) => Equivalent,
            (Item(_), Item(_)) => Equivalent,
            (Quotation(_, ia, oa), Quotation(_, ib, ob)) => {
                // try to catch incompatible quotations
                if let Invalid = ia.borrow().compare(&ib.borrow()) {
                    return Invalid;
                }
                if let Invalid = oa.borrow().compare(&ob.borrow()) {
                    return Invalid;
                }
                // but consider them equivalent in general.
                Equivalent
            }
            (Item(_), Quotation(_, _, _)) => MoreGeneral,
            (Quotation(_, _, _), Item(_)) => MoreSpecific,
            (Row(_), _) => MoreGeneral,
            (_, Row(_)) => MoreSpecific,
        }
    }

    fn substitute(&self, a: &ItemRef, b: &Sequence, quot_stack: &mut Vec<usize>) -> Result<()> {
        match self {
            StackItem::Row(_) => {}
            StackItem::Item(_) => {}
            StackItem::Quotation(name, inps, outs) => {
                let addr = self as *const _ as usize;

                if quot_stack.contains(&addr) { return Ok(()) }

                quot_stack.push(addr);

                inps.try_borrow_mut().map_err(|_| ErrorKind::IncompatibleStackEffects)?.substitute(a, b, quot_stack)?;
                outs.try_borrow_mut().map_err(|_| ErrorKind::IncompatibleStackEffects)?.substitute(a, b, quot_stack)?;

                quot_stack.pop();
            }
        }
        Ok(())
    }
}

enum ItemOrd {
    MoreGeneral,
    Equivalent,
    MoreSpecific,
    Invalid,
}

impl From<ItemOrd> for std::cmp::Ordering {
    fn from(io: ItemOrd) -> Self {
        match io {
            ItemOrd::MoreGeneral => std::cmp::Ordering::Less,
            ItemOrd::Equivalent => std::cmp::Ordering::Equal,
            ItemOrd::MoreSpecific => std::cmp::Ordering::Greater,
            ItemOrd::Invalid => panic!("Invalid something somewhere"),
        }
    }
}

#[derive(Default, Clone)]
pub struct Sequence {
    pub(crate) values: Vec<ItemRef>,
}

impl Sequence {
    pub fn new() -> Self {
        Sequence::default()
    }

    pub fn single(item: ItemRef) -> Self {
        Sequence { values: vec![item] }
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn contains(&self, item: &ItemRef) -> bool {
        self.values.contains(item)
    }

    fn extend(&mut self, other: Sequence) {
        self.values.extend(other.values)
    }

    fn insert(&mut self, idx: usize, item: ItemRef) {
        self.values.insert(idx, item)
    }

    pub fn pop(&mut self) -> Option<ItemRef> {
        self.values.pop()
    }

    pub fn push(&mut self, item: ItemRef) {
        self.values.push(item)
    }

    pub fn front(&self) -> Option<&ItemRef> {
        self.values.get(0)
    }

    pub fn pop_front(&mut self) -> Option<ItemRef> {
        if self.len() == 0 {
            None
        } else {
            Some(self.values.remove(0))
        }
    }

    pub fn into_item(self) -> ItemRef {
        assert_eq!(
            self.len(),
            1,
            "Trying to convert sequence {:?} into an item",
            self
        );
        self.values.into_iter().next().unwrap()
    }

    fn substitute(&mut self, a: &ItemRef, b: &Sequence, quot_stack: &mut Vec<usize>) -> Result<()> {
        let addr = self as *const _ as usize;
        if quot_stack.contains(&addr) { return Ok(()) }

        quot_stack.push(addr);

        let mut i = 0;
        while i < self.len() {
            if &self.values[i] == a {
                let mut tmp = Vec::from(&self.values[..i]);
                tmp.extend(b.values.clone());
                tmp.extend(self.values[i + 1..].iter().cloned());
                self.values = tmp;
                i += b.len();
            } else {
                self.values[i].substitute(a, b, quot_stack)?;
                i += 1;
            }
        }

        quot_stack.pop();        
        Ok(())
    }

    fn compare(&self, other: &Sequence) -> ItemOrd {
        use ItemOrd::*;

        let av = self.values.iter().rev();
        let bv = other.values.iter().rev();

        for (a, b) in av.zip(bv) {
            match a.compare(b) {
                Equivalent => {}
                other => return other,
            }
        }

        if self.len() == other.len() {
            Equivalent
        } else {
            Invalid
        }
    }

    fn match_effects(&self, other: &Sequence) -> Vec<(ItemRef, Sequence)> {
        use StackItem::*;
        let mut subs = vec![];
        for (i, j) in (0..self.len()).rev().zip((0..other.len()).rev()) {
            let a = &self.values[i];
            let b = &other.values[j];
            match (&**a, &**b) {
                (Item(_), Item(_)) => subs.push((a.clone(), Sequence::single(b.clone()))),
                (Item(_), Quotation(_, _, _)) => {
                    subs.push((a.clone(), Sequence::single(b.clone())))
                }
                (Quotation(_, _, _), Item(_)) => {
                    subs.push((b.clone(), Sequence::single(a.clone())))
                }
                (Quotation(_, ref ia, ref oa), Quotation(_, ref ib, ref ob)) => {
                    subs.extend(ia.borrow().match_effects(&ib.borrow()));
                    subs.extend(oa.borrow().match_effects(&ob.borrow()));
                }
                (Row(_), _) => {
                    subs.push((a.clone(), Sequence::from_iter(&other.values[..=j])));
                    break;
                }
                (_, Row(_)) => {
                    subs.push((b.clone(), Sequence::from_iter(&self.values[..=i])));
                    break;
                }
            }
        }
        subs
    }
}

impl std::iter::FromIterator<ItemRef> for Sequence {
    fn from_iter<I: IntoIterator<Item = ItemRef>>(input: I) -> Self {
        Sequence {
            values: input.into_iter().collect(),
        }
    }
}

impl<'a> std::iter::FromIterator<&'a ItemRef> for Sequence {
    fn from_iter<I: IntoIterator<Item = &'a ItemRef>>(input: I) -> Self {
        Sequence {
            values: input.into_iter().cloned().collect(),
        }
    }
}

impl From<ItemRef> for Sequence {
    fn from(item: ItemRef) -> Self {
        Sequence { values: vec![item] }
    }
}

#[derive(Clone)]
pub struct Substitutions {
    subs: HashMap<ItemRef, Sequence>,
}

impl Substitutions {
    fn new() -> Self {
        Substitutions {
            subs: HashMap::new(),
        }
    }

    fn find(&self, item: ItemRef) -> Sequence {
        self.subs
            .get(&item)
            .cloned()
            .unwrap_or_else(|| Sequence::single(item))
    }

    fn add_sequence(&mut self, a: ItemRef, b: Sequence) -> Result<Vec<(ItemRef, Sequence)>> {
        let mut items = vec![];

        if let Some(item) = self.subs.get(&a) {
            items.push(item.clone());
        }

        items.push(Sequence::single(a));
        items.push(b);

        items.sort_unstable_by(|x, y| x.compare(y).into());

        let b = items.pop().unwrap();

        let mut subs = vec![];
        for a in items.into_iter() {
            if a.len() == 1 {
                let a = a.into_item();

                if b.contains(&a) {
                    return Err(ErrorKind::IncompatibleStackEffects.into());
                }

                for other_b in self.subs.values_mut() {
                    other_b.substitute(&a, &b, &mut vec![])?;
                }

                if b.len() == 1 && a == b.values[0] {
                } else {
                    self.subs.insert(a.clone(), b.clone());
                    subs.push((a, b.clone()));
                }
            } else {
                for (a0, b0) in a.match_effects(&b) {
                    subs.extend(self.add_sequence(a0, b0)?);
                }
            }
        }

        if self.has_cycle() {
            Err(ErrorKind::IncompatibleStackEffects.into())
        } else {
            Ok(subs)
        }
    }

    fn has_cycle(&self) -> bool {
        let mut visited: HashMap<_, bool> = self.subs.keys().map(|k| (k, false)).collect();

        let mut queue: Vec<_> = self.subs.keys().collect();
        let mut stack = vec![];

        while let Some(node) = queue.pop() {
            if stack.contains(&node) {
                return true
            }
            match visited.get_mut(node) {
                Some(true) => {
                    stack.pop();
                    continue
                },
                Some(v) => *v = true,
                None => unreachable!(),
            }
            stack.push(node);

            let mut added = false;
            for child in &self.subs[node].values {
                if child == node {
                    // self-substitution is ok
                    continue
                }
                if self.subs.contains_key(child) {
                    queue.push(child);
                    added = true;
                }
                /*if let StackItem::Quotation(_, inp, out) = &**child {
                    for subchild in inp.borrow().values.iter().chain(&out.borrow().values) {
                        if self.subs.contains_key(subchild) {
                            queue.push(subchild);
                            added = true;
                        }
                    }
                }*/
            }
            if !added {
                stack.pop();
            }
        }
        false
    }
}

#[derive(Clone)]
pub struct AbstractStack {
    pub(crate) inputs: Sequence,
    pub(crate) outputs: Sequence,
    pub(crate) subs: Substitutions,
}

impl AbstractStack {
    pub fn new() -> Self {
        let r = [StackItem::anonymous_row()];
        let astack = AbstractStack {
            inputs: Sequence::from_iter(&r),
            outputs: Sequence::from_iter(&r),
            subs: Substitutions::new(),
        };

        astack
    }

    pub fn pop<T: Into<Sequence>>(&mut self, x: T) -> Result<Sequence> {
        self.pop_sequence(x.into())
    }

    pub fn push<T: Into<Sequence>>(&mut self, x: T) {
        self.push_sequence(x.into())
    }

    fn pop_sequence(&mut self, mut targets: Sequence) -> Result<Sequence> {
        let mut result = Sequence::new();
        while let Some(target) = targets.pop() {
            result.extend(self.pop_item(target)?);
        }
        Ok(result)
    }

    fn pop_item(&mut self, target: ItemRef) -> Result<Sequence> {
        if let StackItem::Row(_) = *target {
            let x = replace(&mut self.outputs, Sequence::new());
            self.substitute(&target, &x)?;
            return Ok(x);
        }

        match self.outputs.pop() {
            None => panic!("Abstract Stack Underflow"),
            Some(top) => match *top {
                StackItem::Item(_) => {
                    self.substitute(&target, &Sequence::single(top.clone()))?;
                    Ok(Sequence::single(top))
                }
                StackItem::Row(_) => {
                    self.add_input(target.clone());
                    self.push_item(top);
                    Ok(Sequence::single(target))
                }
                StackItem::Quotation(_, ref ia, ref oa) => {
                    if let StackItem::Quotation(_, ref ib, ref ob) = *target {
                        let mut subs = vec![];
                        subs.extend(ia.borrow().match_effects(&ib.borrow()));
                        subs.extend(oa.borrow().match_effects(&ob.borrow()));

                        for (a, b) in subs {
                            self.substitute(&a, &b)?;
                            target.substitute(&a, &b, &mut vec![])?;
                        }

                        Ok(Sequence::single(target))
                    } else {
                        unimplemented!()
                    }
                }
            },
        }
    }

    fn push_item(&mut self, item: ItemRef) {
        self.outputs.extend(self.subs.find(item));
    }

    fn push_sequence(&mut self, items: Sequence) {
        if items.len() == 1 {
            self.push_item(items.into_item());
        } else {
            self.outputs.extend(items);
        }
    }

    fn add_input(&mut self, item: ItemRef) {
        self.inputs.insert(1, item);
    }

    fn substitute(&mut self, a: &ItemRef, b: &Sequence) -> Result<()> {
        for (a, b) in self.subs.add_sequence(a.clone(), b.clone())? {
            self.inputs.substitute(&a, &b, &mut vec![])?;
            self.outputs.substitute(&a, &b, &mut vec![])?;
        }
        Ok(())
    }

    pub fn apply_effect(&mut self, effect: &StackEffect) -> Result<()> {
        let mut names = HashMap::new();

        for i in effect.inputs.iter().rev().map(NodeRef::borrow) {
            let item = make_item(i, &mut names);
            let item = self.pop(item)?;
            names.insert(i.name().to_string(), item);
        }

        for o in effect.outputs.iter().map(NodeRef::borrow) {
            let item = make_item(o, &mut names);
            self.push(item);
        }
        Ok(())
    }
}

fn make_item(effect: &EffectNode, names: &mut HashMap<String, Sequence>) -> Sequence {
    match effect {
        EffectNode::Item(name) => names
            .entry(name.clone())
            .or_insert_with(|| Sequence::single(StackItem::item(name)))
            .clone(),
        EffectNode::Row(name) => names
            .entry(name.clone())
            .or_insert_with(|| Sequence::single(StackItem::row(name)))
            .clone(),
        EffectNode::Quotation(name, se) => {
            let inputs: Vec<_> = se
                .inputs
                .iter()
                .map(NodeRef::borrow)
                .map(|node| make_item(node, names).into_item())
                .collect();
            let outputs: Vec<_> = se
                .outputs
                .iter()
                .map(NodeRef::borrow)
                .map(|node| make_item(node, names).into_item())
                .collect();
            names
                .entry(name.clone())
                .or_insert_with(|| Sequence::single(StackItem::quot(name, &inputs, &outputs)))
                .clone()
        }
    }
}

impl std::fmt::Debug for AbstractStack {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{:?} -- {:?}     ...with {:?}",
            self.inputs, self.outputs, self.subs
        )
    }
}

impl std::fmt::Display for AbstractStack {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?} -- {:?}", self.inputs, self.outputs)
    }
}

impl std::fmt::Debug for Sequence {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let names: Vec<_> = self.values.iter().map(|x| format!("{:?}", x)).collect();
        write!(f, "{}", names.join(" "))
    }
}

impl std::fmt::Debug for Substitutions {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.subs)
    }
}

impl std::fmt::Debug for StackItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            StackItem::Row(name) => write!(f, "..{}", name),
            StackItem::Item(name) => write!(f, "{}", name),
            StackItem::Quotation(name, a, b) => {
                write!(f, "{}({:?} -- {:?})", name, a.borrow(), b.borrow())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_chaining() {
        let mut astack = AbstractStack::new();
        assert_eq!(format!("{}", astack), ".. -- ..");

        // SWAP (..a x y -- ..a y x)
        let y = astack.pop(StackItem::item("y")).unwrap();
        let x = astack.pop(StackItem::item("x")).unwrap();
        let r = astack.pop(StackItem::row("a")).unwrap();
        astack.push(r);
        astack.push(y);
        astack.push(x);
        assert_eq!(format!("{}", astack), ".. x y -- .. y x");

        // DROP (..b z -- ..b)
        astack.pop(StackItem::item("z")).unwrap();
        let b = astack.pop(StackItem::row("b")).unwrap();
        astack.push(b);
        assert_eq!(format!("{}", astack), ".. x y -- .. y");

        // DROP (z -- )
        astack.pop(StackItem::item("z")).unwrap();
        assert_eq!(format!("{}", astack), ".. x y -- ..");

        // DROP (z -- )
        astack.pop(StackItem::item("z")).unwrap();
        assert_eq!(format!("{}", astack), ".. z x y -- ..");
    }

    #[test]
    fn quotation_chaining() {
        let mut astack = AbstractStack::new();
        assert_eq!(format!("{}", astack), ".. -- ..");

        // SWAP (x y -- y x)
        let y = astack.pop(StackItem::item("y")).unwrap();
        let x = astack.pop(StackItem::item("x")).unwrap();
        astack.push(y);
        astack.push(x);
        assert_eq!(format!("{}", astack), ".. x y -- .. y x");

        // CALL (..a f(..a -- ..b) - ..b)
        let a = StackItem::row("a");
        let b = StackItem::row("b");
        astack
            .pop(StackItem::quot("f", &[a.clone()], &[b.clone()]))
            .unwrap();
        astack.pop(a).unwrap();
        astack.push(b);
        assert_eq!(format!("{}", astack), ".. f(.. y -- ..b) y -- ..b");
    }

    #[test]
    fn quotation_cancelled() {
        let mut astack = AbstractStack::new();

        // [ + ] (-- g(..e x y -- ..e z))
        let e = StackItem::row("e");
        astack.push(StackItem::quot(
            "g",
            &[e.clone(), StackItem::item("x"), StackItem::item("y")],
            &[e, StackItem::item("z")],
        ));
        assert_eq!(format!("{}", astack), ".. -- .. g(..e x y -- ..e z)");

        // CALL (..c f(..c -- ..d) - ..d)
        let c = StackItem::row("c");
        let d = StackItem::row("d");
        astack
            .pop(StackItem::quot("f", &[c.clone()], &[d.clone()]))
            .unwrap();
        astack.pop(c).unwrap();
        astack.push(d);
        assert_eq!(format!("{}", astack), "..e x y -- ..e z");
    }

    #[test]
    fn invalid_chaining() {
        let mut astack = AbstractStack::new();

        // [ + ] (-- g(..e y -- ..e z))
        let e = StackItem::row("e");
        astack.push(StackItem::quot(
            "g",
            &[e.clone(), StackItem::item("y")],
            &[e, StackItem::item("z")],
        ));

        // CALL (..c f(..c i j -- ..d k) - ..d)
        let c = StackItem::row("c");
        if let Err(Error(ErrorKind::IncompatibleStackEffects, _)) = astack.pop(StackItem::quot(
            "f",
            &[c.clone(), StackItem::item("i"), StackItem::item("j")],
            &[c.clone(), StackItem::item("k")],
        )) {

        } else {
            panic!("Expected Error")
        }
    }

    #[test]
    fn invalid_chaining2() {
        let mut astack = AbstractStack::new();

        // [ put ] (-- f(..a -- ..a x))
        let a = StackItem::row("a");
        astack.push(StackItem::quot(
            "f",
            &[a.clone()],
            &[a, StackItem::item("x")],
        ));

        // [ drop ] (-- g(..b y -- ..b))
        let b = StackItem::row("b");
        astack.push(StackItem::quot(
            "g",
            &[b.clone(), StackItem::item("y")],
            &[b],
        ));

        // IF (..c ? h(..c -- ..d) i(..c -- ..d) -- ..d)
        let c = StackItem::row("c");
        let d = StackItem::row("d");
        let h = StackItem::quot(
            "h",
            &[c.clone()],
            &[d.clone()],
        );
        let i = StackItem::quot(
            "i",
            &[c.clone()],
            &[d.clone()],
        );
        match astack.pop(i) {
            Ok(_) => {}
            Err(e) => panic!("Expected Result {:?}", e),
        }
        match astack.pop(h) {
            Ok(_) => panic!("Expected Error"),
            Err(_) => {}
        }
    }
}
