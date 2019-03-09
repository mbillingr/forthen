use forthen_core::errors::*;
use forthen_core::objects::prelude::*;
use forthen_core::State;

/// Load basic operations into the dictionary
pub fn ops(state: &mut State) -> Result<()> {
    // math operations

    state.add_native_word("+", "( a b -- sum )", |state| {
        let b = state.pop()?;
        b.add(state)
    });

    state.add_native_word("-", "( a b -- diff )", |state| {
        let b = state.pop()?;
        let a = state.pop()?;
        state.push((a - b)?)
    });

    state.add_native_word("*", "( a b -- prod )", |state| {
        let b = state.pop()?;
        let a = state.pop()?;
        state.push((a * b)?)
    });

    state.add_native_word("/", "( a b -- quot )", |state| {
        let b = state.pop()?;
        let a = state.pop()?;
        state.push((a / b)?)
    });

    Ok(())
}
