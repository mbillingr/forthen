use forthen_core::errors::*;
use forthen_core::objects::prelude::*;
use forthen_core::State;

/// Load basic operations into the dictionary
pub fn loops(state: &mut State) -> Result<()> {
    state.new_mod("loop".to_string())?;

    state.add_native_word("repeat", "(..x n f(..x -- ..y) -- ..y)", |state| {
        let callee = state.pop()?;
        let n = state.pop_i32()?;
        for _ in 0..n {
            callee.call(state)?;
        }
        Ok(())
    });

    state.add_native_word("for", "(..x a b f(..x i -- ..y) -- ..y)", |state| {
        let callee = state.pop()?;
        let b = state.pop_i32()?;
        let a = state.pop_i32()?;
        for i in a..b {
            state.push(i)?;
            callee.call(state)?;
        }
        Ok(())
    });

    state.add_native_word("while", "(..a cond(..a -- ..b ?) f(..b -- ..c) -- ..c)", |state| {
        let callee = state.pop()?;
        let cond = state.pop()?;
        loop {
            cond.call(state)?;
            if state.pop_bool()? {
                callee.call(state)?;
            } else {
                break
            }
        }
        Ok(())
    });

    state.exit_mod().unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tier0;
    use crate::ops;
    use crate::stack;

    #[test]
    fn repeat() {
        let state = &mut State::new();
        tier0(state).unwrap();
        stack(state).unwrap();
        ops(state).unwrap();
        loops(state).unwrap();

        state.run("USE stack:").unwrap();
        state.run("USE ops:").unwrap();
        state.run("USE loop:").unwrap();

        state.push_str("bottom").unwrap();

        state.run("0 123456 [ 1 + ] repeat").unwrap();
        assert_eq!(123456, state.pop_i32().unwrap());

        assert_eq!("bottom", state.pop_string().unwrap());
    }

    #[test]
    fn for_loop() {
        let state = &mut State::new();
        tier0(state).unwrap();
        stack(state).unwrap();
        ops(state).unwrap();
        loops(state).unwrap();

        state.run("USE stack:").unwrap();
        state.run("USE ops:").unwrap();
        state.run("USE loop:").unwrap();

        state.push_str("bottom").unwrap();

        state.run("1 5 10 [ + ] for").unwrap();
        assert_eq!(1 + 5 + 6 + 7 + 8 + 9, state.pop_i32().unwrap());

        assert_eq!("bottom", state.pop_string().unwrap());
    }

    #[test]
    fn while_loop() {
        let state = &mut State::new();
        tier0(state).unwrap();
        stack(state).unwrap();
        ops(state).unwrap();
        loops(state).unwrap();

        state.run("USE stack:").unwrap();
        state.run("USE ops:").unwrap();
        state.run("USE loop:").unwrap();

        state.push_str("bottom").unwrap();

        state.run("0 128 [ dup 2 <= ] [ 2 / swap 1 + swap ] while").unwrap();
        assert_eq!(1, state.pop_i32().unwrap());
        assert_eq!(7, state.pop_i32().unwrap());

        assert_eq!("bottom", state.pop_string().unwrap());
    }
}

