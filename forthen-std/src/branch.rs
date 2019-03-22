use forthen_core::errors::*;
use forthen_core::objects::prelude::*;
use forthen_core::State;

/// Load basic operations into the dictionary
pub fn branch(state: &mut State) -> Result<()> {
    state.new_mod("branch".to_string())?;

    state.add_native_word(
        "if",
        "(..a ? true(..a -- ..b) false(..a -- ..b) -- ..b)",
        |state| {
            let else_branch = state.pop()?;
            let if_branch = state.pop()?;
            let cond = state.pop()?.try_into_bool()?;
            if cond {
                if_branch.call(state)
            } else {
                else_branch.call(state)
            }
        },
    );

    state.add_native_word("cond", "(..a list -- ..b)", |state| {
        let list = state.pop()?;

        for row in list.as_slice()? {
            match row.as_slice() {
                Ok([cond, action]) => {
                    cond.call(state)?;
                    if state.pop_bool()? {
                        return action.call(state);
                    }
                }
                _ => {
                    return Err(ErrorKind::TypeError(
                        "Expected list of conditions/actions".to_string(),
                    )
                    .into());
                }
            }
        }
        Err(ErrorKind::TypeError("None of the conditions evaluated to true".to_string()).into())
    });

    state.exit_mod().unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops;
    use crate::scope;
    use crate::stack;
    use crate::tier0;

    #[test]
    fn if_word() {
        let state = &mut State::new();
        tier0(state).unwrap();
        branch(state).unwrap();
        scope(state).unwrap();

        state.run("USE branch:").unwrap();
        state.run("USE scope:").unwrap();

        state.add_native_word("true", "( -- b)", |state| state.push(Object::True));
        state.add_native_word("false", "( -- b)", |state| state.push(Object::False));

        state
            .run(":: dup   (x -- x x)   set x get x get x ;")
            .unwrap();
        state
            .run(":: swap   (x y -- y x)   set x set y get x get y ;")
            .unwrap();
        state
            .run(":: over   (a b -- a b a)   set b set a get a get b get a ;")
            .unwrap();
        state
            .run(":: rot   (a b c -- b c a)   set c set b set a get b get c get a  ;")
            .unwrap();
        state.run(":: drop   (x -- )   set x ;").unwrap();

        state.run("123").unwrap(); // push sentinel value on stack

        state.run("false [ \"yes\" ] [ \"no\" ] if").unwrap();
        assert_eq!(state.pop_string().unwrap(), "no");

        state.run("true [ \"yes\" ] [ \"no\" ] if").unwrap();
        assert_eq!(state.pop_string().unwrap(), "yes");

        state
            .run(": yes-or-no ( ? -- s s ) [ \"yes\" dup ] [ \"no\" \"no\" ] if ;")
            .unwrap();

        state.run("false yes-or-no").unwrap();
        assert_eq!(state.pop_string().unwrap(), "no");
        assert_eq!(state.pop_string().unwrap(), "no");

        state.run("true yes-or-no").unwrap();
        assert_eq!(state.pop_string().unwrap(), "yes");
        assert_eq!(state.pop_string().unwrap(), "yes");

        state.assert_pop(123);
    }

    #[test]
    fn cond() {
        let state = &mut State::new();
        tier0(state).unwrap();
        branch(state).unwrap();
        scope(state).unwrap();
        stack(state).unwrap();
        ops(state).unwrap();

        state.run("USE branch:").unwrap();
        state.run("USE scope:").unwrap();
        state.run("USE stack:").unwrap();
        state.run("USE ops:").unwrap();

        state.add_native_word("true", "( -- b)", |state| state.push(Object::True));
        state.add_native_word("false", "( -- b)", |state| state.push(Object::False));

        state
            .run(
                "
            : question (x -- s) [
                    [ [ dup 42 == ] [ drop \"yes, but what is it?\" ] ]
                    [ [ dup 42 < ] [ drop \"too much\" ] ]
                    [ [ 0 < ] [ \"not enough\" ] ]
                ] cond
            ;
        ",
            )
            .unwrap();

        state.assert_run_pop("42 question", &["yes, but what is it?"]);
        state.assert_run_pop("666 question", &["too much"]);
        state.assert_run_pop("1 question", &["not enough"]);
        state.run("-42 question").unwrap_err();
    }
}
