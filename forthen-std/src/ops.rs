use forthen_core::errors::*;
use forthen_core::objects::prelude::*;
use forthen_core::State;
use forthen_core::Object;

/// Load basic operations into the dictionary
pub fn ops(state: &mut State) -> Result<()> {
    state.new_mod("ops".to_string())?;

    state.add_native_word("emit", "(x -- )", |state| {
        match state.pop()? {
            Object::String(s) => print!("{}", s),
            Object::I32(i) => println!("{}", std::char::from_u32(i as u32).unwrap_or('�')),
            obj => return Err(ErrorKind::TypeError(format!("Can't emit {:?}", obj)).into())
        }
        Ok(())
    });

    state.add_native_word("repr", "(x -- s)", |state| state.pop()?.repr(state));

    state.add_native_word("same?", "(a b -- ?)", |state| {
        let b = state.pop()?;
        let a = state.pop()?;
        state.push(b.is_same(&a))
    });

    state.add_native_word("not", "(? -- !?)", |state| state.pop()?.not(state));

    state.add_native_word("==", "(a b -- ?)", |state| state.pop()?.is_eq(state));

    state.add_native_word("!=", "(a b -- ?)", |state| {
        state.pop()?.is_eq(state)?;
        state.pop()?.not(state)
    });

    state.add_native_word("<", "(a b -- ?)", |state| state.pop()?.is_lt(state));
    state.add_native_word(">", "(a b -- ?)", |state| state.pop()?.is_gt(state));
    state.add_native_word("<=", "(a b -- ?)", |state| state.pop()?.is_le(state));
    state.add_native_word(">=", "(a b -- ?)", |state| state.pop()?.is_ge(state));

    state.add_native_word("+", "( a b -- sum )", |state| state.pop()?.add(state));
    state.add_native_word("-", "( a b -- diff )", |state| state.pop()?.sub(state));
    state.add_native_word("*", "( a b -- prod )", |state| state.pop()?.mul(state));
    state.add_native_word("/", "( a b -- quot )", |state| state.pop()?.div(state));

    state.exit_mod().unwrap();

    Ok(())
}
