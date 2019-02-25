use crate::parsing::tokenize;
use std::collections::VecDeque;

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
    Effect(StackEffect),
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

#[derive(Clone, Debug)]
pub struct StackEffect {
    inputs: Vec<StackValue>,
    outputs: Vec<StackValue>,
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
            outputs: vec![StackValue::new(varname)],
        }
    }

    /// simple stack effect of modifying a value on the stack
    pub fn new_mod(varname: &str) -> Self {
        StackEffect {
            inputs: vec![StackValue::new(varname)],
            outputs: vec![StackValue::new(varname)],
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
                se.inputs
                    .last_mut()
                    .expect("Expected name before nested stack effect")
                    .kind = Kind::Effect(effect);
            } else {
                se.inputs.push(StackValue::parse(token));
                input.next();
            }
        }

        assert_eq!(input.next(), Some("--"), "Unexpected end of stack effect");

        while let Some(token) = input.peek() {
            if *token == ")" {
                break;
            } else if *token == "(" {
                let effect = StackEffect::parse_recursive(input);
                se.outputs
                    .last_mut()
                    .expect("Expected name before nested stack effect")
                    .kind = Kind::Effect(effect);
            } else {
                se.outputs.push(StackValue::parse(token));
            }
            input.next();
        }

        assert_eq!(input.next(), Some(")"), "Unexpected end of stack effect");

        se
    }

    fn link_nested_effects(self) -> Self {
        self
        //unimplemented!()
    }

    fn input_values(
        &self,
    ) -> impl DoubleEndedIterator<Item = &StackValue> + ExactSizeIterator<Item = &StackValue> {
        self.inputs.iter()
    }

    fn output_values(
        &self,
    ) -> impl DoubleEndedIterator<Item = &StackValue> + ExactSizeIterator<Item = &StackValue> {
        self.outputs.iter()
    }

    pub fn chain(&self, rhs: &StackEffect) -> Self {
        use Kind::*;

        let mut inputs = VecDeque::from(self.inputs.clone());
        let mut outputs = self.outputs.clone();

        for i in &rhs.inputs {
            let out = match outputs.pop() {
                Some(out) => out,
                None => {
                    inputs.push_front(i.clone());
                    continue
                },
            };

            // todo
            match (&out.kind, &i.kind) {
                (Value, Value) => {}
                (Value, Effect(_)) => {}
                (Value, Unspecified) => {}//outputs.push(out),
                (Unspecified, Value) => unimplemented!(),
                (Unspecified, Effect(_)) => unimplemented!(),
                (Unspecified, Unspecified) => unimplemented!(),
                (Effect(_), Value) => panic!("Error"),
                (Effect(_), Effect(_)) => unimplemented!(),
                (Effect(_), Unspecified) => unimplemented!()
            }
        }

        outputs.extend_from_slice(&rhs.outputs);

        StackEffect {
            inputs: inputs.into(),
            outputs
        }
    }

    pub fn resolve(&mut self, _inputs: &[Option<StackEffect>]) {
        unimplemented!()
    }

    fn format_iter(&self, f: &mut std::fmt::Formatter, iter: impl Iterator<Item=StackValue>) -> std::fmt::Result {
        for val in iter {
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
        self.inputs.len() == rhs.inputs.len() && self.outputs.len() == rhs.outputs.len()
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
                outputs: vec![StackValue::new("b"), StackValue::new("a")],
            }
        );

        let dup = StackEffect::parse("(var -- var var)");

        assert_eq!(
            dup,
            StackEffect {
                inputs: vec![StackValue::new("var")],
                outputs: vec![StackValue::new("var"), StackValue::new("var")],
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
                outputs: vec![StackValue::new("c"), StackValue::new("a"), StackValue::new("b")],
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

        assert_eq!(
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

        //     (--x) (--y) (a b c -- b c a) (..a f(..a -- ..b) -- ..b) = (-- x y f) (..a f(..a -- ..b) -- ..b)

        let new = StackEffect::parse("( -- x)");
        let rot = StackEffect::parse("(a b c -- b c a)");
        let call = StackEffect::parse("(..a func(..a -- ..b) -- ..b)");

        println!("{}", new.chain(&new));
        println!("{}", new.chain(&new).chain(&rot));
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
