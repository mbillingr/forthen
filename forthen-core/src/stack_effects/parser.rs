use super::effect::StackEffect;
use super::element::{Element, ElementRef};
use super::scratchpad::Scratchpad;
use crate::errors::*;
use std::ops::Deref;

pub fn parse_effect<'a>(
    scratchpad: &mut Scratchpad,
    input: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
) -> Result<StackEffect> {
    let mut inputs = parse_sequence(scratchpad, input, "--")?;
    let mut outputs = parse_sequence(scratchpad, input, ")")?;

    let i0 = inputs.get(0).map(|i| i.borrow().is_ellipsis());
    let o0 = outputs.get(0).map(|i| i.borrow().is_ellipsis());

    match (i0, o0) {
        (Some(true), _) | (_, Some(true)) => {}
        _ => {
            let id = scratchpad.insert(Element::Ellipsis("_".to_string()));

            let mut tmp = vec![id.clone()];
            tmp.extend(inputs);
            inputs = tmp;

            let mut tmp = vec![id];
            tmp.extend(outputs);
            outputs = tmp;
        }
    }

    // todo: I don't think it's necessary to normalize here because the parser should not generate nested sequences
    Ok(StackEffect::new(inputs, outputs).normalized())
}

fn parse_quotation<'a>(
    scratchpad: &mut Scratchpad,
    input: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
    name: &str,
) -> Result<Element> {
    let se = parse_effect(scratchpad, input)?;
    Ok(Element::Callable(name.to_string(), se))
}

fn parse_sequence<'a>(
    scratchpad: &mut Scratchpad,
    input: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
    terminator: &str,
) -> Result<Vec<ElementRef>> {
    let mut sequence = vec![];
    while let Some(token) = input.next() {
        if token == terminator {
            return Ok(sequence);
        }

        let element = if let Some(&"(") = input.peek() {
            parse_quotation(scratchpad, input, token)?
        } else if token.starts_with("..") {
            Element::Ellipsis(token[2..].to_string())
        } else {
            Element::Item(token.to_string())
        };

        let id = scratchpad.update(element.clone());
        use std::collections::HashSet;
        sequence.push(id);
    }

    Err(ErrorKind::EndOfInput.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::tokenize;

    #[test]
    fn parse_swap() {
        let scrpad = &mut Scratchpad::default();
        let swap = parse_effect(scrpad, &mut tokenize("(a b -- b a)").peekable()).unwrap();

        assert_eq!(
            StackEffect::new(vec![
                    scrpad.find_by_name("_").unwrap().clone(),
                    scrpad.find_by_name("a").unwrap().clone(),
                    scrpad.find_by_name("b").unwrap().clone()
                ],
                vec![
                    scrpad.find_by_name("_").unwrap().clone(),
                    scrpad.find_by_name("b").unwrap().clone(),
                    scrpad.find_by_name("a").unwrap().clone()
                ],
            ),
            swap
        );
    }

    #[test]
    fn parse_dup() {
        let scrpad = &mut Scratchpad::default();
        let dup = parse_effect(scrpad, &mut tokenize("(var -- var var)").peekable()).unwrap();

        let r = scrpad.find_by_name("_").unwrap().clone();
        let var = scrpad.find_by_name("var").unwrap().clone();

        assert_eq!(
            StackEffect::new(
                vec![r.clone(), var.clone()],
                vec![r.clone(), var.clone(), var.clone()],
            ),
            dup
        );
    }

    #[test]
    fn parse_call() {
        let scrpad = &mut Scratchpad::default();
        let call = parse_effect(
            scrpad,
            &mut tokenize("(..a f(..a -- ..b) -- ..b)").peekable(),
        )
        .unwrap();

        let a = scrpad.find_by_name("a").unwrap().clone();
        let b = scrpad.find_by_name("b").unwrap().clone();
        let f = scrpad.find_by_name("f").unwrap().clone();

        if let Element::Callable(_, _) = *f.borrow() {
        } else {
            panic!("f should be a callable")
        };

        assert_eq!(
            StackEffect::new(vec![a, f], vec![b]),
            call
        );
    }

    #[test]
    fn parse_recursion() {
        let scrpad = &mut Scratchpad::default();
        let rec = parse_effect(scrpad, &mut tokenize("(..a f(..a f -- ) -- )").peekable()).unwrap();

        let a = scrpad.find_by_name("a").unwrap().clone();
        let f = scrpad.find_by_name("f").unwrap().clone();

        let fx = StackEffect::new(vec![a, f.clone()], vec![]);

        assert_eq!(fx, rec);

        if let Element::Callable(_, ref f_sub) = *f.borrow() {
            assert_eq!(&fx, f_sub);
        } else {
            panic!("{:?} is not callable", f)
        };
    }
}
