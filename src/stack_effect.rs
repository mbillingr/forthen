use crate::parsing::tokenize;
use std::collections::{HashMap, VecDeque};

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

#[derive(Debug, PartialEq, Clone)]
enum Kind {
    Value,
    Effect(SubEffect),
    Unspecified,
}

#[derive(Debug, PartialEq, Clone)]
struct StackValue {
    name: String,
    kind: Kind,
}

impl StackValue {
    fn new(name: &str) -> Self {
        StackValue {
            name: name.to_string(),
            kind: Kind::Value,
        }
    }

    fn parse(token: &str) -> Self {
        if token.starts_with("..") {
            StackValue {
                name: token[2..].to_string(),
                kind: Kind::Unspecified,
            }
        } else {
            StackValue::new(token)
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct SubEffect {
    inputs: Vec<usize>,
    outputs: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct StackEffect {
    values: Vec<StackValue>,
    inputs: Vec<usize>,
    outputs: Vec<usize>,
}

impl StackEffect {
    pub fn new() -> Self {
        StackEffect {
            values: vec![],
            inputs: vec![],
            outputs: vec![],
        }
    }

    /// simple stack effect of pushing a value on the stack
    pub fn new_pushing(varname: &str) -> Self {
        StackEffect {
            values: vec![StackValue::new(varname)],
            inputs: vec![],
            outputs: vec![0],
        }
    }

    /// simple stack effect of modifying a value on the stack
    pub fn new_mod(varname: &str) -> Self {
        StackEffect {
            values: vec![StackValue::new(varname)],
            inputs: vec![0],
            outputs: vec![0],
        }
    }

    pub fn parse(input: &str) -> Self {
        StackEffect::parse_recursive(&mut tokenize(input).peekable()).link_nested_effects()
    }

    pub fn parse_recursive<'a>(
        input: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
    ) -> Self {
        let mut se = StackEffect::new();

        assert!(input.next().expect("Unexpected end of stack effect") == "(");

        while let Some(token) = input.peek() {
            if *token == "--" {
                break;
            }
            if *token == "(" {
                let effect = StackEffect::parse_recursive(input);
                let iv = *se
                    .inputs
                    .last()
                    .expect("Expected name before nested stack effect");
                se.values[iv].kind = Kind::Effect(se.convert_subeffect(effect));
            } else {
                assert!(
                    se.input_values().all(|val| val.name != *token),
                    "Stack effects inputs must have unique names"
                );
                se.inputs.push(se.values.len());
                se.values.push(StackValue::parse(token));
                input.next();
            }
        }

        assert_eq!(input.next(), Some("--"), "Unexpected end of stack effect");

        while let Some(token) = input.peek() {
            if *token == ")" {
                break;
            } else if *token == "(" {
                let effect = StackEffect::parse_recursive(input);
                let ov = *se
                    .outputs
                    .last()
                    .expect("Expected name before nested stack effect");
                se.values[ov].kind = Kind::Effect(se.convert_subeffect(effect));
            } else if let Some(pos) = se.find_value(token) {
                se.outputs.push(pos);
            } else {
                se.outputs.push(se.values.len());
                se.values.push(StackValue::parse(token));
            }
            input.next();
        }

        assert_eq!(input.next(), Some(")"), "Unexpected end of stack effect");

        se
    }

    fn convert_subeffect(&mut self, se: StackEffect) -> SubEffect {
        let mut inputs = vec![];
        let mut outputs = vec![];

        for i in se.input_values() {
            if let Some(pos) = self.find_value(&i.name) {
                inputs.push(pos);
            } else {
                inputs.push(self.values.len());
                self.values.push(i.clone());
            }
        }

        for o in se.output_values() {
            if let Some(pos) = self.find_value(&o.name) {
                outputs.push(pos);
            } else {
                outputs.push(self.values.len());
                self.values.push(o.clone());
            }
        }

        SubEffect {
            inputs, outputs
        }
    }

    fn link_nested_effects(self) -> Self {
        self
        //unimplemented!()
    }

    fn input_values(
        &self,
    ) -> impl DoubleEndedIterator<Item = &StackValue> + ExactSizeIterator<Item = &StackValue> {
        self.inputs.iter().map(move |&iv| &self.values[iv])
    }

    fn output_values(
        &self,
    ) -> impl DoubleEndedIterator<Item = &StackValue> + ExactSizeIterator<Item = &StackValue> {
        self.outputs.iter().map(move |&iv| &self.values[iv])
    }

    fn find_value(&self, name: &str) -> Option<usize> {
        self.values.iter().position(|val| val.name == name)
    }

    pub fn chain(&self, rhs: &StackEffect) -> Self {
        let mut stack = AbstractStack::new();
        stack.apply_effect(self);
        stack.apply_effect(rhs);
        stack.into_effect()
    }

    pub fn resolve(&mut self, _inputs: &[Option<StackEffect>]) {
        unimplemented!()
        /*for (val, j) in self.inputs.iter()
            .map(|&i| &self.values[i])
            .filter(|val| val.kind != Kind::Unspecified)
            .zip(inputs)
        {
            match (&val.kind, j) {
                (Kind::Unspecified, _) => unreachable!(),
                (Kind::Value, _) => {},
                (Kind::Effect(_), None) => panic!("expected quotation"),
                (Kind::Effect(ref template), Some(ref actual)) => {
                    // stack effect: (..a func(..a -- ..b) -- ..b)
                    //  with inputs: [(x y -- z)]  ->  ..a := xy; ..b := z  ->  (x y -- z)

                    // : apply   ( -- x y)   20 10
                    // : apply   (f -- x y f)   20 10 rot
                    // : apply   (..a f(..a x y -- ..b) -- ..b)   20 10 rot call
                    unimplemented!()
                },
            }
        }*/
    }

    fn format_iter(&self, f: &mut std::fmt::Formatter, iter: impl Iterator<Item=usize>) -> std::fmt::Result {
        for idx in iter {
            let val = &self.values[idx];
            match val.kind {
                Kind::Value => write!(f, " {}", val.name)?,
                Kind::Unspecified => write!(f, " ..{}", val.name)?,
                Kind::Effect(ref se) => {
                    write!(f, " {}(", val.name)?;
                    self.format_iter(f, se.inputs.iter().cloned())?;
                    write!(f, " --")?;
                    self.format_iter(f, se.outputs.iter().cloned())?;
                    write!(f, " )")?;
                },
            }
        }
        Ok(())
    }
}

impl std::cmp::PartialEq for StackEffect {
    fn eq(&self, rhs: &Self) -> bool {
        let n = self.inputs.len();
        let m = self.outputs.len();

        if n != rhs.inputs.len() {
            return false;
        }
        if m != rhs.outputs.len() {
            return false;
        }

        let mut a = StackEffectRealization::from_stack_effect(self);
        let mut b = StackEffectRealization::from_stack_effect(rhs);

        for i in 0..n {
            a.set_input(i, i);
            b.set_input(i, i);
        }

        for o in 0..m {
            if a.get_output(o) != b.get_output(o) {
                return false;
            }
        }

        true
    }
}

impl std::fmt::Display for StackEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(")?;
        self.format_iter(f, self.inputs.iter().cloned())?;
        write!(f, " --")?;
        self.format_iter(f, self.outputs.iter().cloned())?;
        write!(f, " )")
    }
}

struct StackEffectRealization<'a, T> {
    se: &'a StackEffect,
    values: Vec<Option<T>>,
}

impl<'a, T: Clone> StackEffectRealization<'a, T> {
    fn from_stack_effect(se: &'a StackEffect) -> Self {
        StackEffectRealization {
            se,
            values: vec![None; se.values.len()],
        }
    }

    fn set_input(&mut self, i: usize, val: T) {
        let idx = self.se.inputs[i];
        assert!(self.values[idx].is_none());
        self.values[idx] = Some(val);
    }

    fn set_output(&mut self, i: usize, val: T) {
        let idx = self.se.outputs[i];
        assert!(self.values[idx].is_none());
        self.values[idx] = Some(val);
    }

    fn get_output(&self, o: usize) -> Option<&T> {
        let idx = self.se.outputs[o];
        self.values[idx].as_ref()
    }
}

#[derive(Debug, Copy, Clone)]
enum AbstractValue {
    New(usize),
    Input(usize),
}

#[derive(Debug)]
struct AbstractStack {
    values: Vec<StackValue>,
    inputs: VecDeque<usize>,
    outputs: Vec<AbstractValue>,
}

impl AbstractStack {
    fn new() -> Self {
        AbstractStack {
            values: vec![],
            inputs: VecDeque::new(),
            outputs: vec![],
        }
    }

    fn pop(&mut self, val: StackValue) -> AbstractValue {
        match self.outputs.pop() {
            Some(x) => x,
            None => {
                let v = self.values.len();
                self.values.push(val);

                let i = self.inputs.len();
                self.inputs.push_front(v);

                AbstractValue::Input(i)
            }
        }
    }

    fn push(&mut self, x: AbstractValue) {
        self.outputs.push(x);
    }

    fn push_new(&mut self, x: StackValue) {
        let v = self.values.len();
        self.values.push(x);

        self.push(AbstractValue::New(v));
    }

    fn apply_effect(&mut self, se: &StackEffect) {
        let mut ser = StackEffectRealization::from_stack_effect(se);

        // pop inputs from stack
        for (i, val) in se.input_values().cloned().enumerate().rev() {
            ser.set_input(i, self.pop(val));
        }

        // push outputs on stack
        for (o, val) in se.output_values().cloned().enumerate() {
            match ser.get_output(o) {
                Some(x) => self.push(*x),
                None => {
                    self.push_new(val);
                    ser.set_output(o, *self.outputs.last().unwrap());
                }
            }
        }
    }

    fn into_effect(mut self) -> StackEffect {
        self.resolve_name_conflicts();

        let mut se = StackEffect::new();

        se.outputs = self
            .outputs
            .iter()
            .map(|out| match out {
                AbstractValue::Input(i) => self.inputs[self.inputs.len() - 1 - i],
                AbstractValue::New(v) => *v,
            })
            .collect();
        se.inputs = self.inputs.into_iter().collect();

        se.values = self.values;

        se
    }

    fn resolve_name_conflicts(&mut self) {
        let mut name_counts = HashMap::new();

        for val in &mut self.values {
            let c = name_counts.entry(val.name.clone()).or_insert(0);
            if *c == 0 {
                *c += 1;
            } else {
                val.name += &format!("{}", c);
                *c += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
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
    }

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
        assert_eq!(swap.chain(&swap), StackEffect::parse("(a b -- a b)"));
        assert_eq!(dup.chain(&dup), StackEffect::parse("(x -- x x x)"));
        assert_eq!(drop.chain(&drop), StackEffect::parse("(b a -- )"));
        assert_eq!(put.chain(&put), StackEffect::parse("(a b -- c d a b)"));

        assert_eq!(dup.chain(&drop), StackEffect::parse("(x -- x)"));
        assert_ne!(dup.chain(&drop), StackEffect::parse("(a b -- a b)"));

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

        let new = StackEffect::parse("( -- x)");
        let rot = StackEffect::parse("(a b c -- b c a)");
        let call = StackEffect::parse("(..a func(..a -- ..b) -- ..b)");

        println!("{}", new.chain(&new).chain(&rot).chain(&call));

        assert_eq!(new.chain(&new), StackEffect::parse("( -- x y)"));
        assert_eq!(new.chain(&new).chain(&rot), StackEffect::parse("(f -- x y f)"));
        assert_eq!(new.chain(&new).chain(&rot).chain(&call), StackEffect::parse("(..a f(..a x y -- ..b) -- ..b)"));

    }

    #[test]
    fn dynamic_effect() {
        let sfx = StackEffect::parse("(..a ? yes(..a ? -- ..b) no(..a -- ..b) -- ..b)");
        println!("{:?}", sfx);
        println!("{}", sfx);
        panic!()
    }
}
