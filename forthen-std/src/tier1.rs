use forthen_core::errors::Result;
use forthen_core::objects::prelude::*;
use forthen_core::State;

/// Load language tier 1 into the dictionary
///
/// Tier 1 contains low level native words that form the basic building blocks of the language
pub fn tier1(state: &mut State) -> Result<()> {
    // development tools
    state.add_native_word(".s", "( -- )", |state| {
        println!("{:?}", state.stack);
        Ok(())
    });
    state.add_native_word(".", "( x -- )", |state| {
        state.pop()?.repr(state)?;
        println!("{}", state.pop()?.try_into_rc_string()?);
        Ok(())
    });

    // stack operations
    state.add_native_word("dup", "(a -- a a)", State::dup);
    state.add_native_word("drop", "(a -- )", |state| state.pop().map(|_| ()));
    state.add_native_word("swap", "(a b -- b a)", State::swap);
    state.add_native_word("over", "(a b -- a b a)", State::over);
    state.add_native_word("rot", "(a b c -- b c a)", State::rot);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_ops() {
        let state = &mut State::new();
        tier1(state).unwrap();

        state.run("123 456 swap").unwrap();
        state.assert_stack(&[456, 123]);

        state.clear_stack();

        state.run("\"abc\" \"def\" \"ghi\" swap").unwrap();
        state.assert_stack(&["abc", "ghi", "def"]);

        state.clear_stack();

        state.run("123 \"abc\" swap").unwrap();
        assert_eq!(state.pop().unwrap(), 123);
        assert_eq!(state.pop().unwrap(), "abc");
    }
}
