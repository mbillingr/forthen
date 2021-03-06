use super::branch::branch;
use super::list::list;
use super::loops::loops;
use super::ops::ops;
use super::scope::scope;
use super::stack::stack;
use super::table::table;
use super::tier0::tier0;
use super::timeit::timeit;
use forthen_core::{Result, State};

/// Load basic operations into the dictionary
pub fn stdlib(state: &mut State) -> Result<()> {
    tier0(state)?;

    state.new_mod("std".to_string())?;

    branch(state)?;
    list(state)?;
    loops(state)?;
    ops(state)?;
    scope(state)?;
    stack(state)?;
    table(state)?;
    timeit(state)?;

    state.run(
        "
        USE branch:
        USE list:
        USE loop:
        USE ops:
        USE scope:
        USE stack:
        USE table:
    ",
    )?;

    state.exit_mod().unwrap();

    Ok(())
}
