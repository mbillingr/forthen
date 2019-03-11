use super::element::ElementRef;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct StackEffect {
    pub(crate) inputs: Vec<ElementRef>,
    pub(crate) outputs: Vec<ElementRef>,
}

impl StackEffect {
    pub fn simplified(&self) -> StackEffect {
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
}

impl std::fmt::Display for StackEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let simple = self.simplified();

        let a: Vec<_> = simple.inputs.iter().map(|x| format!("{:?}", x)).collect();
        let b: Vec<_> = simple.outputs.iter().map(|x| format!("{:?}", x)).collect();

        write!(f, "{} -- {}", a.join(" "), b.join(" "))
    }
}
