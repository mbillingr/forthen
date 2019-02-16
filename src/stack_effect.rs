use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

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

#[derive(Debug, Clone)]
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
        match out {
            OutputValue::New(val) => val.name == name,
            OutputValue::Input(i) => self.inputs[*i].name == name,
            OutputValue::RepeatedOutput(o) => self.is_output_name(&self.outputs[*o], name),
        }
    }

    pub fn chain(self, rhs: StackEffect) -> Self {
        // todo: I'm sure this function could be implemented *much* better...

        let mut stack = AbstractStack::new();

        let mut inputs1 = VecDeque::new();
        for inp in self.inputs.into_iter().rev() {
            inputs1.push_front(stack.pop(inp));
        }

        let out0 = stack.len();

        for out in self.outputs {
            match out {
                OutputValue::New(val) => stack.push_new(val),
                OutputValue::Input(i) => stack.push(inputs1[i].clone()),
                OutputValue::RepeatedOutput(o) => {
                    let x = stack.get(out0 + o).clone();
                    stack.push(x);
                }
            }
        }

        let mut inputs2 = VecDeque::new();
        for inp in rhs.inputs.into_iter().rev() {
            inputs2.push_front(stack.pop(inp));
        }

        let out0 = stack.len();

        for out in rhs.outputs {
            match out {
                OutputValue::New(val) => stack.push_new(val),
                OutputValue::Input(i) => stack.push(inputs2[i].clone()),
                OutputValue::RepeatedOutput(o) => {
                    let x = stack.get(out0 + o).clone();
                    stack.push(x);
                }
            }
        }

        let inputs: Vec<_> = stack
            .original
            .iter()
            .map(|x| match **x {
                AbstractValue::Input(ref v, _) => v.clone(),
                AbstractValue::New(_) => panic!("unexpected new value in inputs"),
            })
            .collect();

        let mut seen = HashMap::new();
        let mut outputs = vec![];
        for out in stack.stack.iter() {
            let ptr = &(**out) as *const _;

            if seen.contains_key(&ptr) {
                outputs.push(OutputValue::RepeatedOutput(seen[&ptr]));
                continue;
            }

            seen.insert(ptr, outputs.len());
            match **out {
                AbstractValue::New(ref val) => outputs.push(OutputValue::New(val.clone())),
                AbstractValue::Input(_, i) => {
                    outputs.push(OutputValue::Input(inputs.len() - i - 1))
                }
            }
        }

        StackEffect { inputs, outputs }
    }
}

impl std::cmp::PartialEq for StackEffect {
    fn eq(&self, rhs: &Self) -> bool {
        self.inputs.len() == rhs.inputs.len() && self.outputs == rhs.outputs
    }
}

#[derive(Debug, Hash, Eq, PartialEq)]
enum AbstractValue<T> {
    Input(T, usize),
    New(T),
}

struct AbstractStack {
    values: Vec<Rc<AbstractValue<StackValue>>>,
    original: VecDeque<Rc<AbstractValue<StackValue>>>,
    stack: Vec<Rc<AbstractValue<StackValue>>>,
}

impl AbstractStack {
    fn new() -> Self {
        AbstractStack {
            values: vec![],
            original: VecDeque::new(),
            stack: vec![],
        }
    }

    fn len(&self) -> usize {
        self.stack.len()
    }

    fn pop(&mut self, val: StackValue) -> Rc<AbstractValue<StackValue>> {
        match self.stack.pop() {
            Some(x) => x,
            None => {
                let x = Rc::new(AbstractValue::Input(val, self.original.len()));
                self.values.push(x.clone());
                self.original.push_front(x.clone());
                x
            }
        }
    }

    fn push(&mut self, x: Rc<AbstractValue<StackValue>>) {
        self.stack.push(x);
    }

    fn push_new(&mut self, x: StackValue) {
        let x = Rc::new(AbstractValue::New(x));
        self.values.push(x.clone());
        self.push(x);
    }

    fn get(&mut self, i: usize) -> Rc<AbstractValue<StackValue>> {
        self.stack[i].clone()
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
    }
}
