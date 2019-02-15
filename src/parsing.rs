pub fn tokenize(input: &str) -> impl Iterator<Item = &str> {
    let mut it = input.char_indices().peekable();

    std::iter::repeat(())
        .map(move |_| {
            skip_while(&mut it, char::is_whitespace);

            match it.peek() {
                None => return None,
                Some((_, '"')) => {
                    let (a, _) = it.next().unwrap();
                    skip_while(&mut it, |ch| ch != '"');
                    it.next();
                    match it.peek() {
                        Some((b, _)) => return Some(&input[a..*b]),
                        None => return Some(&input[a..]),
                    }
                }
                Some((i, _)) => {
                    let a = *i;
                    skip_while(&mut it, |ch| !ch.is_whitespace());
                    match it.peek() {
                        Some((b, _)) => return Some(&input[a..*b]),
                        None => return Some(&input[a..]),
                    }
                }
            }
        })
        .take_while(Option::is_some)
        .map(Option::unwrap)
}

fn skip_while(
    it: &mut std::iter::Peekable<std::str::CharIndices>,
    predicate: impl Fn(char) -> bool,
) {
    while let Some(true) = it.peek().map(|&(_, ch)| predicate(ch)) {
        it.next();
    }
}
