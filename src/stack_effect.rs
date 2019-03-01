use crate::abstract_stack::{AbstractStack, Sequence, StackItem};
use crate::parsing::tokenize;
use std::collections::{HashMap, HashSet};

pub trait IntoStackEffect {
    fn into_stack_effect(self) -> StackEffect;
}

impl IntoStackEffect for StackEffect {
    fn into_stack_effect(self) -> StackEffect {
        self
    }
}

impl IntoStackEffect for &str {
    fn into_stack_effect(self) -> StackEffect {
        StackEffect::parse(self)
    }
}

impl IntoStackEffect for String {
    fn into_stack_effect(self) -> StackEffect {
        StackEffect::parse(&self)
    }
}

#[derive(Debug, Clone)]
pub struct StackEffect {
    pub(crate) inputs: Vec<EffectNode>,
    pub(crate) outputs: Vec<EffectNode>,
}

impl StackEffect {
    pub fn new() -> Self {
        StackEffect {
            inputs: vec![EffectNode::Row("_".to_string())],
            outputs: vec![EffectNode::Row("_".to_string())],
        }
    }

    pub fn new_pushing(varname: &str) -> Self {
        StackEffect {
            inputs: vec![EffectNode::Row("_".to_string())],
            outputs: vec![EffectNode::Row("_".to_string()), EffectNode::Item(varname.to_string())],
        }
    }

    pub fn new_quotation(name: &str, effect: StackEffect) -> Self {
        StackEffect {
            inputs: vec![EffectNode::Row("_".to_string())],
            outputs: vec![EffectNode::Row("_".to_string()), EffectNode::quoted_effect(name, effect)],
        }
    }

    pub fn new_mod(varname: &str) -> Self {
        StackEffect {
            inputs: vec![EffectNode::Row("_".to_string()), EffectNode::Item(varname.to_string())],
            outputs: vec![EffectNode::Row("_".to_string()), EffectNode::Item(varname.to_string())],
        }
    }

    pub fn parse(input: &str) -> Self {
        parse_effect(&mut tokenize(input).peekable())
    }

    pub fn chain(&self, rhs: &Self) -> Self {
        let (a, b) = rename_effects(self, rhs);
        
        //println!("({}) ({})", a, b);

        let mut astack = AbstractStack::new();
        astack.apply_effect(&a).unwrap();
        astack.apply_effect(&b).unwrap();

        //println!("    ->  {:?}", astack);

        StackEffect {
            inputs: astack.inputs.into(),
            outputs: astack.outputs.into(),
        }
    }

    fn all_names(&self) -> HashSet<&str> {
        let mut names = HashSet::new();
        names.extend(self.inputs.iter().flat_map(|i| i.all_names()));
        names.extend(self.outputs.iter().flat_map(|o| o.all_names()));
        names
    }

    fn renamed(&self, mapping: &HashMap<&str, String>) -> StackEffect {
        StackEffect {
            inputs: self.inputs.iter().map(|i| i.renamed(mapping)).collect(),
            outputs: self.outputs.iter().map(|o| o.renamed(mapping)).collect(),
        }
    }

    fn simplified(&self) -> StackEffect {
        let mut inputs = self.inputs.clone();
        let mut outputs = self.outputs.clone();

        loop {
            match (inputs.first(), outputs.first()) {
                (None, _) | (_, None) => break,
                (Some(a), Some(b)) if !a.is_same(b) => break,
                (Some(a), Some(b)) if a.is_same(b) => {
                    if outputs[1..].iter().any(|b| a.is_same(b)) {
                        break;
                    }
                }
                _ => {}
            }

            inputs.remove(0);
            outputs.remove(0);
        }

        StackEffect {
            inputs, outputs
        }
    }
}

fn rename_effects(left: &StackEffect, right: &StackEffect) -> (StackEffect, StackEffect) {
    let left_names = left.all_names();
    let right_names = right.all_names();
    let dups: HashSet<&str> = left_names.intersection(&right_names).cloned().collect();

    let left_rename: HashMap<_, _> = left_names
        .into_iter()
        .map(|name| {
            (
                name,
                if dups.contains(&name) {
                    add_to_name(name, 0)
                } else {
                    name.to_string()
                },
            )
        })
        .collect();

    let right_rename: HashMap<_, _> = right_names
        .into_iter()
        .map(|name| {
            (
                name,
                if dups.contains(&name) {
                    add_to_name(name, 1)
                } else {
                    name.to_string()
                },
            )
        })
        .collect();

    (left.renamed(&left_rename), right.renamed(&right_rename))
}

fn add_to_name(name: &str, n: i32) -> String {
    let i = name.len() - name.chars().rev().filter(|ch| ch.is_digit(10)).count();

    let number: i32 = name[i..].parse().unwrap_or(0);

    format!("{}{}", &name[..i], number + n)
}

fn compare_sequence<'a>(
    seq_a: &'a [EffectNode],
    seq_b: &'a [EffectNode],
    pos_a: &mut HashMap<&'a str, usize>,
    pos_b: &mut HashMap<&'a str, usize>,
) -> bool {
    for (a, b) in seq_a.iter().zip(seq_b) {
        let n = pos_a.len();
        let m = pos_b.len();
        let i = pos_a.entry(a.name()).or_insert(n);
        let j = pos_b.entry(b.name()).or_insert(m);

        if i != j || a != b {
            return false;
        }
    }
    true
}

fn compare_effects(
    in_a: &[EffectNode],
    out_a: &[EffectNode],
    in_b: &[EffectNode],
    out_b: &[EffectNode],
) -> bool {
    let mut self_pos = HashMap::new();
    let mut other_pos = HashMap::new();

    if !compare_sequence(in_a, in_b, &mut self_pos, &mut other_pos) {
        return false;
    }
    compare_sequence(out_a, out_b, &mut self_pos, &mut other_pos)
}

impl std::cmp::PartialEq for StackEffect {
    fn eq(&self, other: &Self) -> bool {
        compare_effects(&self.inputs, &self.outputs, &other.inputs, &other.outputs)
    }
}

impl std::fmt::Display for StackEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let simple = self.simplified();

        let a: Vec<_> = simple.inputs.iter().map(|x| format!("{:?}", x)).collect();
        let b: Vec<_> = simple.outputs.iter().map(|x| format!("{:?}", x)).collect();

        write!(f, "{} -- {}", a.join(" "), b.join(" "))
    }
}

#[derive(Clone)]
pub(crate) enum EffectNode {
    Row(String),
    Item(String),
    Quotation(String, Vec<EffectNode>, Vec<EffectNode>),
}

impl EffectNode {
    pub fn quoted_effect(name: &str, se: StackEffect) -> Self {
        EffectNode::Quotation("!".to_owned(), se.inputs, se.outputs).prefixed(name)
    }

    pub fn name(&self) -> &str {
        match self {
            EffectNode::Row(name) | EffectNode::Item(name) | EffectNode::Quotation(name, _, _) => {
                name
            }
        }
    }

    fn is_same(&self, other: &Self) -> bool {
        use EffectNode::*;
        match (self, other) {
            (Row(na), Row(nb)) |
            (Item(na), Item(nb)) |
            (Quotation(na, _, _), Quotation(nb, _, _)) => na == nb,
            _ => false
        }
    }

    fn all_names(&self) -> HashSet<&str> {
        let mut names = HashSet::new();

        match self {
            EffectNode::Row(name) | EffectNode::Item(name) => {
                names.insert(name.as_str());
            }
            EffectNode::Quotation(name, a, b) => {
                names.insert(name);
                names.extend(a.iter().flat_map(|i| i.all_names()));
                names.extend(b.iter().flat_map(|o| o.all_names()));
            }
        }

        names
    }

    fn renamed(&self, mapping: &HashMap<&str, String>) -> EffectNode {
        match self {
            EffectNode::Row(name) => EffectNode::Row(mapping[name.as_str()].clone()),
            EffectNode::Item(name) => EffectNode::Item(mapping[name.as_str()].clone()),
            EffectNode::Quotation(name, a, b) => EffectNode::Quotation(
                mapping[name.as_str()].clone(),
                a.iter().map(|i| i.renamed(mapping)).collect(),
                b.iter().map(|o| o.renamed(mapping)).collect(),
            ),
        }
    }

    fn prefixed(self, prefix: &str) -> Self {
        match self {
            EffectNode::Row(name) => EffectNode::Row(format!("{}{}", prefix, name)),
            EffectNode::Item(name) => EffectNode::Item(format!("{}{}", prefix, name)),
            EffectNode::Quotation(name, a, b) => EffectNode::Quotation(
                format!("{}{}", prefix, name),
                a.into_iter().map(|i| i.prefixed(prefix)).collect(),
                b.into_iter().map(|o| o.prefixed(prefix)).collect(),
            ),
        }
    }

    fn simplified(&self) -> Self {
        match self {
            EffectNode::Row(_) |
            EffectNode::Item(_) => self.clone(),
            EffectNode::Quotation(name, inputs, outputs) => {
                let mut inputs = inputs.clone();
                let mut outputs = outputs.clone();

                loop {
                    match (inputs.first(), outputs.first()) {
                        (None, _) | (_, None) => break,
                        (Some(a), Some(b)) if !a.is_same(b) => break,
                        (Some(a), Some(b)) if a.is_same(b) => {
                            if outputs[1..].iter().any(|b| a.is_same(b)) {
                                break;
                            }
                        }
                        _ => {}
                    }

                    inputs.remove(0);
                    outputs.remove(0);
                }

                EffectNode::Quotation(name.clone(), inputs, outputs)
            }
        }
    }
}

impl From<Sequence> for Vec<EffectNode> {
    fn from(seq: Sequence) -> Self {
        seq.values
            .iter()
            .map(|x| match **x {
                StackItem::Item(ref name) => EffectNode::Item(name.clone()),
                StackItem::Row(ref name) => EffectNode::Row(name.clone()),
                StackItem::Quotation(ref name, ref a, ref b) => EffectNode::Quotation(
                    name.clone(),
                    a.clone().into_inner().into(),
                    b.clone().into_inner().into(),
                ),
            })
            .collect()
    }
}

impl std::cmp::PartialEq for EffectNode {
    fn eq(&self, other: &Self) -> bool {
        use EffectNode::*;
        match (self, other) {
            (Row(_), Row(_)) => true,
            (Item(_), Item(_)) => true,
            (Quotation(_, ia, oa), Quotation(_, ib, ob)) => {
                let mut a_pos = HashMap::new();
                let mut b_pos = HashMap::new();
                if !compare_sequence(ia, ib, &mut a_pos, &mut b_pos) {
                    return false;
                }
                compare_sequence(oa, ob, &mut a_pos, &mut b_pos)
            }
            _ => false,
        }
    }
}

impl std::fmt::Debug for EffectNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.simplified() {
            EffectNode::Row(name) => write!(f, "..{}", name),
            EffectNode::Item(name) => write!(f, "{}", name),
            EffectNode::Quotation(name, a, b) => {
                let a: Vec<_> = a.iter().map(|x| format!("{:?}", x)).collect();
                let b: Vec<_> = b.iter().map(|x| format!("{:?}", x)).collect();
                write!(f, "{}({} -- {})", name, a.join(" "), b.join(" "))
            }
        }
    }
}

fn parse_effect<'a>(input: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>) -> StackEffect {
    assert_eq!(input.next(), Some("("));
    let mut inputs = parse_sequence(input, "--");
    let mut outputs = parse_sequence(input, ")");

    match (inputs.get(0), outputs.get(0)) {
        (Some(EffectNode::Row(_)), _) => {}
        (_, Some(EffectNode::Row(_))) => {}
        _ => {
            let mut tmp = vec![EffectNode::Row("_".to_string())];
            tmp.extend(inputs);
            inputs = tmp;
            
            let mut tmp = vec![EffectNode::Row("_".to_string())];
            tmp.extend(outputs);
            outputs = tmp;
        }
    }

    StackEffect { inputs, outputs }
}

fn parse_quotation<'a>(
    input: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
    name: &str,
) -> EffectNode {
    assert_eq!(input.next(), Some("("));
    let mut inputs = parse_sequence(input, "--");
    let mut outputs = parse_sequence(input, ")");

    match (inputs.get(0), outputs.get(0)) {
        (Some(EffectNode::Row(_)), _) => {}
        (_, Some(EffectNode::Row(_))) => {}
        _ => {
            let mut tmp = vec![EffectNode::Row("_".to_string())];
            tmp.extend(inputs);
            inputs = tmp;
            
            let mut tmp = vec![EffectNode::Row("_".to_string())];
            tmp.extend(outputs);
            outputs = tmp;
        }
    }

    EffectNode::Quotation(name.to_string(), inputs, outputs)
}

fn parse_sequence<'a>(
    input: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
    terminator: &str,
) -> Vec<EffectNode> {
    let mut sequence = vec![];
    while let Some(token) = input.next() {
        if token == terminator {
            return sequence;
        }

        let element = if let Some(&"(") = input.peek() {
            parse_quotation(input, token)
        } else if token.starts_with("..") {
            EffectNode::Row(token[2..].to_string())
        } else {
            EffectNode::Item(token.to_string())
        };

        sequence.push(element);
    }
    panic!("Unexpected end of input")
}

#[cfg(test)]
mod tests {
    use super::*;

    /*#[test]
    fn parse_effects() {
        let swap = StackEffect::parse("(a b -- b a)");

        assert_eq!(
            swap,
            StackEffect {
                values: vec![StackValue::new("a"), StackValue::new("b")],
                inputs: vec![0, 1],
                outputs: vec![1, 0],
            }
        );

        let dup = StackEffect::parse("(var -- var var)");

        assert_eq!(
            dup,
            StackEffect {
                values: vec![StackValue::new("var")],
                inputs: vec![0],
                outputs: vec![0, 0],
            }
        );

        let drop = StackEffect::parse("(x -- )");

        assert_eq!(
            drop,
            StackEffect {
                values: vec![StackValue::new("x")],
                inputs: vec![0],
                outputs: vec![],
            }
        );

        let put = StackEffect::parse("(a b -- c a b)");

        assert_eq!(
            put,
            StackEffect {
                values: vec![
                    StackValue::new("a"),
                    StackValue::new("b"),
                    StackValue::new("c")
                ],
                inputs: vec![0, 1],
                outputs: vec![2, 0, 1],
            }
        );

        assert_eq!(
            put,
            StackEffect {
                values: vec![
                    StackValue::new("b"),
                    StackValue::new("c"),
                    StackValue::new("a")
                ],
                inputs: vec![2, 0],
                outputs: vec![1, 2, 0],
            }
        );
    }*/

    #[test]
    fn equivalence_effects() {
        assert_eq!(StackEffect::parse("( -- )"), StackEffect::parse("(--)"));
        assert_eq!(
            StackEffect::parse("(b -- b)"),
            StackEffect::parse("(a -- a)")
        );
        assert_eq!(
            StackEffect::parse("(x y -- y x)"),
            StackEffect::parse("(a b -- b a)")
        );

        assert_ne!(
            StackEffect::parse("(a b -- a a)"),
            StackEffect::parse("(a b -- b b)")
        );
        assert_eq!(
            StackEffect::parse("(a b -- c)"),
            StackEffect::parse("(b a -- z)")
        );

        assert_eq!(
            StackEffect::parse("( -- a b)"),
            StackEffect::parse("( -- b a)")
        );
        assert_eq!(
            StackEffect::parse("(b -- a b b c)"),
            StackEffect::parse("(b -- c b b a)")
        );
    }

    #[test]
    fn chain_effects() {
        let new = StackEffect::parse("( -- x)");
        let swap = StackEffect::parse("(a b -- b a)");
        let dup = StackEffect::parse("(var -- var var)");
        let drop = StackEffect::parse("(x -- )");
        let put = StackEffect::parse("(a b -- c a b)");

        let drop3 = StackEffect::parse("(a b c -- )");

        assert_eq!(new.chain(&new), StackEffect::parse("( -- x y)"));
        assert_eq!(swap.chain(&swap), StackEffect::parse("( -- )"));
        assert_eq!(dup.chain(&dup), StackEffect::parse("(x -- x x x)"));
        assert_eq!(drop.chain(&drop), StackEffect::parse("(b a -- )"));
        assert_eq!(put.chain(&put), StackEffect::parse("(a b -- c d a b)"));

        assert_eq!(dup.chain(&drop), StackEffect::parse("( -- )"));

        assert_eq!(swap.chain(&put), StackEffect::parse("(a b -- c b a)"));
        assert_eq!(put.chain(&swap), StackEffect::parse("(a b -- c b a)"));

        assert_eq!(dup.chain(&drop).chain(&drop), StackEffect::parse("(x --)"));

        assert_eq!(put, StackEffect::parse("(a b -- c a b)"));
        assert_eq!(put.chain(&swap), StackEffect::parse("(a b -- c b a)"));
        assert_eq!(
            put.chain(&swap).chain(&drop),
            StackEffect::parse("(a b -- c b)")
        );
        assert_eq!(
            put.chain(&swap).chain(&drop).chain(&dup),
            StackEffect::parse("(a b -- c b b)")
        );
        assert_eq!(
            put.chain(&swap).chain(&drop).chain(&dup).chain(&new),
            StackEffect::parse("(a b -- c b b d)")
        );
        assert_eq!(
            put.chain(&swap)
                .chain(&drop)
                .chain(&dup)
                .chain(&new)
                .chain(&swap),
            StackEffect::parse("(a b -- c b d b)")
        );

        assert_eq!(drop3.chain(&swap), StackEffect::parse("(a b c d e -- b a)"));
        assert_eq!(swap.chain(&drop3), StackEffect::parse("(c a b -- )"));
    }

    #[test]
    fn regression_input_mapping() {
        let sfx = StackEffect::new_pushing("z")
            .chain(&StackEffect::new_pushing("z"))
            .chain(&StackEffect::parse("(x c b a -- x)"));
        assert_eq!(sfx, StackEffect::parse("(x c -- x)"));
    }

    #[test]
    fn chain_unspec() {
        // Correct chaining of sub effect templates... e.g. what should happen in the case where
        // we have a word that creates and passes arguments to a quotation it recieves as input?
        // For example, we have a word that takes as input a quotation such as [ + ] or [ - ] and
        // applies it to 20 and 10?
        // Considering that we do not know what quotation will be actually passed (we could get [ + + ]),
        // I think the resulting stack effect should look like this (third row):
        //     : apply   ( -- x y)   20 10
        //     : apply   (f -- x y f)   20 10 rot
        //     : apply   (..a f(..a x y -- ..b) -- ..b)   20 10 rot call

        //     (-- x) (-- y) (a b c -- b c a) (..a f(..a -- ..b) -- ..b)
        //                     = (f -- x y f) (..a f(..a -- ..b) -- ..b)
        //                                  = (..a f(..a x y -- ..b) -- ..b)

        let new = StackEffect::parse("( -- x)");
        let rot = StackEffect::parse("(a b c -- b c a)");
        let call = StackEffect::parse("(..a func(..a -- ..b) -- ..b)");

        assert_eq!(new.chain(&new), StackEffect::parse("( -- x y)"));
        assert_eq!(
            new.chain(&new).chain(&rot),
            StackEffect::parse("(f -- x y f)")
        );
        assert_eq!(
            new.chain(&new).chain(&rot).chain(&call),
            StackEffect::parse("(..a f(..a x y -- ..b) -- ..b)")
        );
    }

    #[test]
    fn if_effect() {
        let sfx = StackEffect::parse("(..a ? yes(..a -- ..b) no(..a -- ..b) -- ..b)");
        let yes = StackEffect::parse("(..d -- ..d f(..c -- ..c x))");
        let no = StackEffect::parse("(..d -- ..d f(..c -- ..c x))");

        println!("{}", yes.chain(&no).chain(&sfx));
        assert_eq!(
            yes.chain(&no).chain(&sfx),
            StackEffect::parse("(cond -- value)")
        );
    }
}
