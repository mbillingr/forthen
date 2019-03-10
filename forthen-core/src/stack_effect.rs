use crate::abstract_stack::{AbstractStack, Sequence, StackItem};
use crate::errors::*;
use crate::parsing::tokenize;
use std::collections::{HashMap, HashSet};

pub trait IntoStackEffect: Sized {
    fn try_into_stack_effect(self) -> Result<StackEffect>;

    fn into_stack_effect(self) -> StackEffect {
        self.try_into_stack_effect().unwrap()
    }
}

impl IntoStackEffect for StackEffect {
    fn try_into_stack_effect(self) -> Result<StackEffect> {
        Ok(self)
    }
}

impl IntoStackEffect for &str {
    fn try_into_stack_effect(self) -> Result<StackEffect> {
        StackEffect::parse(self)
    }
}

impl IntoStackEffect for String {
    fn try_into_stack_effect(self) -> Result<StackEffect> {
        StackEffect::parse(&self)
    }
}

#[derive(Debug, Default, Clone)]
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
            outputs: vec![
                EffectNode::Row("_".to_string()),
                EffectNode::Item(varname.to_string()),
            ],
        }
    }

    pub fn new_quotation(name: &str, effect: StackEffect) -> Self {
        StackEffect {
            inputs: vec![EffectNode::Row("_".to_string())],
            outputs: vec![
                EffectNode::Row("_".to_string()),
                EffectNode::quoted_effect(name, effect),
            ],
        }
    }

    pub fn new_mod(varname: &str) -> Self {
        StackEffect {
            inputs: vec![
                EffectNode::Row("_".to_string()),
                EffectNode::Item(varname.to_string()),
            ],
            outputs: vec![
                EffectNode::Row("_".to_string()),
                EffectNode::Item(varname.to_string()),
            ],
        }
    }

    pub fn parse(input: &str) -> Result<Self> {
        parse_effect(&mut tokenize(input).peekable()).map_err(|e| e)
    }

    pub fn chain(&self, rhs: &Self) -> Result<Self> {
        let (a, b) = rename_effects(self, rhs);

        //println!("({}) ({})", a, b);

        let mut astack = AbstractStack::new();
        astack.apply_effect(&a)?;
        astack.apply_effect(&b)?;

        //println!("    ->  {:?}", astack);

        Ok(StackEffect {
            inputs: astack.inputs.into(),
            outputs: astack.outputs.into(),
        })
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

        StackEffect { inputs, outputs }
    }

    fn prefixed(self, prefix: &str) -> Self {
        StackEffect {
            inputs: self
                .inputs
                .into_iter()
                .map(|i| i.prefixed(prefix))
                .collect(),
            outputs: self
                .outputs
                .into_iter()
                .map(|o| o.prefixed(prefix))
                .collect(),
        }
    }
}

fn rename_effects(left: &StackEffect, right: &StackEffect) -> (StackEffect, StackEffect) {
    let left_names = left.all_names();
    let right_names = right.all_names();
    let dups: HashSet<&str> = left_names.intersection(&right_names).cloned().collect();

    let mut used_names: HashSet<String> = left_names
        .union(&right_names)
        .map(|s| s.to_string())
        .collect();

    let left_rename: HashMap<_, _> = left_names
        .into_iter()
        .map(|name| {
            (
                name,
                if dups.contains(&name) {
                    let mut rename = add_to_name(&name, 0);
                    while used_names.contains(rename.as_str()) {
                        rename = add_to_name(&rename, 1)
                    }
                    used_names.insert(rename.clone());
                    rename
                } else {
                    used_names.insert(name.to_string());
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
                    let mut rename = add_to_name(&name, 0);
                    while used_names.contains(rename.as_str()) {
                        rename = add_to_name(&rename, 1)
                    }
                    used_names.insert(rename.clone());
                    rename
                } else {
                    used_names.insert(name.to_string());
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
    if seq_a.len() != seq_b.len() {
        return false;
    }
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
        let a = self.simplified();
        let b = other.simplified();
        compare_effects(&a.inputs, &a.outputs, &b.inputs, &b.outputs)
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
    Quotation(String, StackEffect),
}

impl EffectNode {
    pub fn quoted_effect(name: &str, se: StackEffect) -> Self {
        EffectNode::Quotation("!".to_owned(), se).prefixed(name)
    }

    pub fn name(&self) -> &str {
        match self {
            EffectNode::Row(name) | EffectNode::Item(name) | EffectNode::Quotation(name, _) => name,
        }
    }

    fn is_same(&self, other: &Self) -> bool {
        use EffectNode::*;
        match (self, other) {
            (Row(na), Row(nb)) | (Item(na), Item(nb)) | (Quotation(na, _), Quotation(nb, _)) => {
                na == nb
            }
            _ => false,
        }
    }

    fn all_names(&self) -> HashSet<&str> {
        let mut names = HashSet::new();

        match self {
            EffectNode::Row(name) | EffectNode::Item(name) => {
                names.insert(name.as_str());
            }
            EffectNode::Quotation(name, se) => {
                names.insert(name);
                names.extend(se.all_names());
            }
        }

        names
    }

    fn renamed(&self, mapping: &HashMap<&str, String>) -> EffectNode {
        match self {
            EffectNode::Row(name) => EffectNode::Row(mapping[name.as_str()].clone()),
            EffectNode::Item(name) => EffectNode::Item(mapping[name.as_str()].clone()),
            EffectNode::Quotation(name, se) => {
                EffectNode::Quotation(mapping[name.as_str()].clone(), se.renamed(mapping))
            }
        }
    }

    fn prefixed(self, prefix: &str) -> Self {
        match self {
            EffectNode::Row(name) => EffectNode::Row(format!("{}{}", prefix, name)),
            EffectNode::Item(name) => EffectNode::Item(format!("{}{}", prefix, name)),
            EffectNode::Quotation(name, se) => {
                EffectNode::Quotation(format!("{}{}", prefix, name), se.prefixed(prefix))
            }
        }
    }

    fn simplified(&self) -> Self {
        match self {
            EffectNode::Row(_) | EffectNode::Item(_) => self.clone(),
            EffectNode::Quotation(name, se) => EffectNode::Quotation(name.clone(), se.simplified()),
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
                    StackEffect {
                        inputs: a.clone().into_inner().into(),
                        outputs: b.clone().into_inner().into(),
                    },
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
            (Quotation(_, sea), Quotation(_, seb)) => sea == seb,
            _ => false,
        }
    }
}

impl std::fmt::Debug for EffectNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.simplified() {
            EffectNode::Row(name) => write!(f, "..{}", name),
            EffectNode::Item(name) => write!(f, "{}", name),
            EffectNode::Quotation(name, se) => write!(f, "{}({})", name, se),
        }
    }
}

fn parse_effect<'a>(
    input: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
) -> Result<StackEffect> {
    assert_eq!(input.next(), Some("("));
    let mut inputs = parse_sequence(input, "--")?;
    let mut outputs = parse_sequence(input, ")")?;

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

    Ok(StackEffect { inputs, outputs })
}

fn parse_quotation<'a>(
    input: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
    name: &str,
) -> Result<EffectNode> {
    let se = parse_effect(input)?;
    Ok(EffectNode::Quotation(name.to_string(), se))
}

fn parse_sequence<'a>(
    input: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
    terminator: &str,
) -> Result<Vec<EffectNode>> {
    let mut sequence = vec![];
    while let Some(token) = input.next() {
        if token == terminator {
            return Ok(sequence);
        }

        let element = if let Some(&"(") = input.peek() {
            parse_quotation(input, token)?
        } else if token.starts_with("..") {
            EffectNode::Row(token[2..].to_string())
        } else {
            EffectNode::Item(token.to_string())
        };

        sequence.push(element);
    }

    Err(ErrorKind::EndOfInput.into())
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
        assert_eq!(
            StackEffect::parse("( -- )").unwrap(),
            StackEffect::parse("(--)").unwrap()
        );
        assert_eq!(
            StackEffect::parse("(b -- b)").unwrap(),
            StackEffect::parse("(a -- a)").unwrap()
        );
        assert_eq!(
            StackEffect::parse("(x y -- y x)").unwrap(),
            StackEffect::parse("(a b -- b a)").unwrap()
        );
        assert_ne!(
            StackEffect::parse("(a b -- a a)").unwrap(),
            StackEffect::parse("(a b -- b b)").unwrap()
        );
        assert_eq!(
            StackEffect::parse("(a b -- c)").unwrap(),
            StackEffect::parse("(b a -- z)").unwrap()
        );
        assert_eq!(
            StackEffect::parse("( -- a b)").unwrap(),
            StackEffect::parse("( -- b a)").unwrap()
        );
        assert_eq!(
            StackEffect::parse("(b -- a b b c)").unwrap(),
            StackEffect::parse("(b -- c b b a)").unwrap()
        );
        assert_ne!(
            StackEffect::parse("(a b -- b a b a)").unwrap(),
            StackEffect::parse("(x -- y)").unwrap()
        );
    }

    #[test]
    fn chain_effects() {
        let new = "( -- x)".into_stack_effect();
        let swap = "(a b -- b a)".into_stack_effect();
        let dup = "(var -- var var)".into_stack_effect();
        let drop = "(x -- )".into_stack_effect();
        let put = "(a b -- c a b)".into_stack_effect();

        let drop3 = "(a b c -- )".into_stack_effect();

        assert_eq!(new.chain(&new).unwrap(), "( -- x y)".into_stack_effect());
        assert_eq!(swap.chain(&swap).unwrap(), "( -- )".into_stack_effect());
        assert_eq!(dup.chain(&dup).unwrap(), "(x -- x x x)".into_stack_effect());
        assert_eq!(drop.chain(&drop).unwrap(), "(b a -- )".into_stack_effect());
        assert_eq!(
            put.chain(&put).unwrap(),
            "(a b -- c d a b)".into_stack_effect()
        );

        assert_eq!(dup.chain(&drop).unwrap(), "( -- )".into_stack_effect());

        assert_eq!(
            swap.chain(&put).unwrap(),
            "(a b -- c b a)".into_stack_effect()
        );
        assert_eq!(
            put.chain(&swap).unwrap(),
            "(a b -- c b a)".into_stack_effect()
        );

        assert_eq!(
            dup.chain(&drop).unwrap().chain(&drop).unwrap(),
            "(x --)".into_stack_effect()
        );

        assert_eq!(put, "(a b -- c a b)".into_stack_effect());
        assert_eq!(
            put.chain(&swap).unwrap(),
            "(a b -- c b a)".into_stack_effect()
        );
        assert_eq!(
            put.chain(&swap).unwrap().chain(&drop).unwrap(),
            "(a b -- c b)".into_stack_effect()
        );
        assert_eq!(
            put.chain(&swap)
                .unwrap()
                .chain(&drop)
                .unwrap()
                .chain(&dup)
                .unwrap(),
            "(a b -- c b b)".into_stack_effect()
        );
        assert_eq!(
            put.chain(&swap)
                .unwrap()
                .chain(&drop)
                .unwrap()
                .chain(&dup)
                .unwrap()
                .chain(&new)
                .unwrap(),
            "(a b -- c b b d)".into_stack_effect()
        );
        assert_eq!(
            put.chain(&swap)
                .unwrap()
                .chain(&drop)
                .unwrap()
                .chain(&dup)
                .unwrap()
                .chain(&new)
                .unwrap()
                .chain(&swap)
                .unwrap(),
            "(a b -- c b d b)".into_stack_effect()
        );

        assert_eq!(
            drop3.chain(&swap).unwrap(),
            "(a b c d e -- b a)".into_stack_effect()
        );
        assert_eq!(
            swap.chain(&drop3).unwrap(),
            "(c a b -- )".into_stack_effect()
        );
    }

    #[test]
    fn regression_input_mapping() {
        let sfx = StackEffect::new_pushing("z")
            .chain(&StackEffect::new_pushing("z"))
            .unwrap()
            .chain(&"(x c b a -- x)".into_stack_effect())
            .unwrap();
        assert_eq!(sfx, "(x c -- x)".into_stack_effect());
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

        let new = "( -- x)".into_stack_effect();
        let rot = "(a b c -- b c a)".into_stack_effect();
        let call = "(..a func(..a -- ..b) -- ..b)".into_stack_effect();

        assert_eq!(new.chain(&new).unwrap(), "( -- x y)".into_stack_effect());
        assert_eq!(
            new.chain(&new).unwrap().chain(&rot).unwrap(),
            "(f -- x y f)".into_stack_effect()
        );
        assert_eq!(
            new.chain(&new)
                .unwrap()
                .chain(&rot)
                .unwrap()
                .chain(&call)
                .unwrap(),
            "(..a f(..a x y -- ..b) -- ..b)".into_stack_effect()
        );
    }

    #[test]
    fn if_effect() {
        let sfx = "(..a ? yes(..a -- ..b) no(..a -- ..b) -- ..b)".into_stack_effect();
        let yes = "(..d -- ..d f(..c -- ..c x))".into_stack_effect();
        let no = "(..d -- ..d f(..c -- ..c x))".into_stack_effect();
        let put = "(..d -- ..d f(..c -- ..c x))".into_stack_effect();
        let drop = "(..d -- ..d f(..c x -- ..c))".into_stack_effect();

        assert_eq!(
            yes.chain(&no).unwrap().chain(&sfx).unwrap(),
            "(cond -- value)".into_stack_effect()
        );

        assert_eq!(
            drop.chain(&drop).unwrap().chain(&sfx).unwrap(),
            "(x ? -- )".into_stack_effect()
        );

        assert_eq!(
            put.chain(&put).unwrap().chain(&sfx).unwrap(),
            "(? -- x)".into_stack_effect()
        );

        // todo: these return some weird stack effects
        //       actually they should error out because the two if branches have incompatible
        //       effects

        assert_eq!(
            put.chain(&drop).unwrap().chain(&sfx).unwrap(),
            "(? -- x)".into_stack_effect()
        );

        assert_eq!(
            drop.chain(&put).unwrap().chain(&sfx).unwrap(),
            "(? -- x)".into_stack_effect()
        );
    }

    #[test]
    fn multiple_renaming() {
        let store = "( x -- )".into_stack_effect();
        let fetch = "( -- x )".into_stack_effect();
        let two_dup = store
            .chain(&store)
            .unwrap()
            .chain(&fetch)
            .unwrap()
            .chain(&fetch)
            .unwrap()
            .chain(&fetch)
            .unwrap()
            .chain(&fetch)
            .unwrap();

        assert_eq!(two_dup, "(a b -- c d e f)".into_stack_effect());
    }
}
