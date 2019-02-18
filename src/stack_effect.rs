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

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct StackValue {
    name: String,
}

impl StackValue {
    fn new(name: &str) -> Self {
        StackValue {
            name: name.to_string(),
        }
    }

    fn parse(token: &str) -> Self {
        StackValue::new(token)
    }
}

#[derive(Debug, Clone)]
enum OutputValue {
    New(StackValue),
    Input(usize),
    RepeatedOutput(usize),
}

impl std::cmp::PartialEq for OutputValue {
    fn eq(&self, rhs: &Self) -> bool {
        use OutputValue::*;
        match (self, rhs) {
            (New(_), New(_)) => true,
            (Input(a), Input(b)) => a == b,
            (RepeatedOutput(a), RepeatedOutput(b)) => a == b,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StackEffect {
    inputs: Vec<StackValue>,
    outputs: Vec<OutputValue>,
}

impl StackEffect {
    pub fn new() -> Self {
        StackEffect {
            inputs: vec![],
            outputs: vec![],
        }
    }

    /// simple stack effect of pushing a value on the stack
    pub fn new_pushing(varname: &str) -> Self {
        StackEffect {
            inputs: vec![],
            outputs: vec![OutputValue::New(StackValue::new(varname))],
        }
    }

    /// simple stack effect of modifying a value on the stack
    pub fn new_mod(varname: &str) -> Self {
        StackEffect {
            inputs: vec![StackValue::new(varname)],
            outputs: vec![OutputValue::Input(0)],
        }
    }

    pub fn parse(input: &str) -> Self {
        let mut se = StackEffect::new();

        assert!(input.starts_with('('));
        assert!(input.ends_with(')'));

        let mut input = input[1..input.len() - 1].trim().split_whitespace();

        while let Some(token) = input.next() {
            if token == "--" {
                break;
            }
            assert!(se.inputs.iter().all(|val| val.name != token)); // make sure input names are unique
            se.inputs.push(StackValue::parse(token));
        }

        while let Some(token) = input.next() {
            if let Some(pos) = se.find_output(token) {
                se.outputs.push(OutputValue::RepeatedOutput(pos));
            } else if let Some(pos) = se.find_input(token) {
                se.outputs.push(OutputValue::Input(pos));
            } else {
                se.outputs.push(OutputValue::New(StackValue::parse(token)));
            }
        }

        se
    }

    fn find_input(&self, name: &str) -> Option<usize> {
        self.inputs.iter().position(|val| val.name == name)
    }

    fn find_output(&self, name: &str) -> Option<usize> {
        self.outputs
            .iter()
            .position(|out| self.is_output_name(out, name))
    }

    fn is_output_name(&self, out: &OutputValue, name: &str) -> bool {
        self.get_output_name(out) == name
    }

    fn get_output_name<'a>(&'a self, out: &'a OutputValue) -> &'a str {
        match out {
            OutputValue::New(val) => &val.name,
            OutputValue::Input(i) => &self.inputs[*i].name,
            OutputValue::RepeatedOutput(o) => self.get_output_name(&self.outputs[*o]),
        }
    }

    pub fn chain(self, rhs: StackEffect) -> Self {
        let mut stack = AbstractStack::new();
        stack.apply_effect(self);
        stack.apply_effect(rhs);
        stack.into_effect()
    }
}

impl std::cmp::PartialEq for StackEffect {
    fn eq(&self, rhs: &Self) -> bool {
        self.inputs.len() == rhs.inputs.len() && self.outputs == rhs.outputs
    }
}

impl std::fmt::Display for StackEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(")?;
        for i in &self.inputs {
            write!(f, " {}", i.name)?;
        }
        write!(f, " --")?;
        for o in &self.outputs {
            write!(f, " {}", self.get_output_name(o))?;
        }
        write!(f, " )")
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

    fn len(&self) -> usize {
        self.outputs.len()
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

    fn get(&mut self, i: usize) -> AbstractValue {
        self.outputs[i]
    }

    fn apply_effect(&mut self, se: StackEffect) {
        // pop inputs from stack
        let mut inputs = VecDeque::new();
        for val in se.inputs.into_iter().rev() {
            inputs.push_front(self.pop(val));
        }

        let offset = self.len();

        // push outputs on stack
        for out in se.outputs {
            match out {
                OutputValue::New(val) => self.push_new(val),
                OutputValue::Input(i) => self.push(inputs[i]),
                OutputValue::RepeatedOutput(o) => {
                    let x = self.get(offset + o);
                    self.push(x);
                }
            }
        }
    }

    fn into_effect(self) -> StackEffect {
        let mut name_counts = HashMap::new();

        let mut values: Vec<_> = self
            .values
            .into_iter()
            .map(|mut val| {
                let c = name_counts.entry(val.name.clone()).or_insert(0);
                if *c == 0 {
                    *c += 1;
                    val
                } else {
                    val.name += &format!("{}", c);
                    *c += 1;
                    val
                }
            })
            .map(|val| OutputValue::New(val))
            .collect();

        let mut se = StackEffect::new();

        //let ni = self.inputs.len();

        se.inputs = self
            .inputs
            .iter()
            .rev()
            .enumerate()
            .map(
                |(i, &v)| match std::mem::replace(&mut values[v], OutputValue::Input(i)) {
                    OutputValue::New(val) => val,
                    _ => unreachable!(),
                },
            )
            .rev()
            .collect();

        for out in self.outputs {
            let v = match out {
                AbstractValue::Input(i) => self.inputs[i],
                AbstractValue::New(v) => v,
            };

            let val = if let OutputValue::RepeatedOutput(o) = values[v] {
                OutputValue::RepeatedOutput(o)
            } else {
                std::mem::replace(
                    &mut values[v],
                    OutputValue::RepeatedOutput(se.outputs.len()),
                )
            };

            se.outputs.push(val);
        }

        se
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
                inputs: vec![StackValue::new("a"), StackValue::new("b")],
                outputs: vec![OutputValue::Input(1), OutputValue::Input(0)],
            }
        );

        let dup = StackEffect::parse("(var -- var var)");

        assert_eq!(
            dup,
            StackEffect {
                inputs: vec![StackValue::new("var")],
                outputs: vec![OutputValue::Input(0), OutputValue::RepeatedOutput(0)],
            }
        );

        let drop = StackEffect::parse("(x -- )");

        assert_eq!(
            drop,
            StackEffect {
                inputs: vec![StackValue::new("x")],
                outputs: vec![],
            }
        );

        let put = StackEffect::parse("(a b -- c a b)");

        assert_eq!(
            put,
            StackEffect {
                inputs: vec![StackValue::new("a"), StackValue::new("b")],
                outputs: vec![
                    OutputValue::New(StackValue::new("c")),
                    OutputValue::Input(0),
                    OutputValue::Input(1)
                ],
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

        assert_eq!(
            new.clone().chain(new.clone()),
            StackEffect::parse("( -- x y)")
        );
        assert_eq!(
            swap.clone().chain(swap.clone()),
            StackEffect::parse("(a b -- a b)")
        );
        assert_eq!(
            dup.clone().chain(dup.clone()),
            StackEffect::parse("(x -- x x x)")
        );
        assert_eq!(
            drop.clone().chain(drop.clone()),
            StackEffect::parse("(b a -- )")
        );
        assert_eq!(
            put.clone().chain(put.clone()),
            StackEffect::parse("(a b -- c d a b)")
        );

        assert_eq!(
            dup.clone().chain(drop.clone()),
            StackEffect::parse("(x -- x)")
        );
        assert_ne!(
            dup.clone().chain(drop.clone()),
            StackEffect::parse("(a b -- a b)")
        );

        assert_eq!(
            swap.clone().chain(put.clone()),
            StackEffect::parse("(a b -- c b a)")
        );
        assert_eq!(
            put.clone().chain(swap.clone()),
            StackEffect::parse("(a b -- c b a)")
        );

        assert_eq!(
            dup.clone().chain(drop.clone()).chain(drop.clone()),
            StackEffect::parse("(x --)")
        );

        assert_eq!(put, StackEffect::parse("(a b -- c a b)"));
        assert_eq!(
            put.clone().chain(swap.clone()),
            StackEffect::parse("(a b -- c b a)")
        );
        assert_eq!(
            put.clone().chain(swap.clone()).chain(drop.clone()),
            StackEffect::parse("(a b -- c b)")
        );
        assert_eq!(
            put.clone()
                .chain(swap.clone())
                .chain(drop.clone())
                .chain(dup.clone()),
            StackEffect::parse("(a b -- c b b)")
        );
        assert_eq!(
            put.clone()
                .chain(swap.clone())
                .chain(drop.clone())
                .chain(dup.clone())
                .chain(new.clone()),
            StackEffect::parse("(a b -- c b b d)")
        );
        assert_eq!(
            put.clone()
                .chain(swap.clone())
                .chain(drop.clone())
                .chain(dup.clone())
                .chain(new.clone())
                .chain(swap.clone()),
            StackEffect::parse("(a b -- c b d b)")
        );

        assert_eq!(
            drop3.clone().chain(swap.clone()),
            StackEffect::parse("(a b c d e -- b a)")
        );
        assert_eq!(
            swap.clone().chain(drop3.clone()),
            StackEffect::parse("(c a b -- )")
        );
    }

    #[test]
    fn regression_input_mapping() {
        let sfx = StackEffect::new_pushing("z")
            .chain(StackEffect::new_pushing("z"))
            .chain(StackEffect::parse("(x c b a -- x)"));
        assert_eq!(sfx, StackEffect::parse("(x c -- x)"));
    }
}
