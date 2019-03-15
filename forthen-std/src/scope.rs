use forthen_core::errors::*;
use forthen_core::CompilerScope;
use forthen_core::Object;
use forthen_core::State;
use forthen_core::{ByteCode, Opcode};
use std::rc::Rc;

/// Load language tier 0 into the dictionary
///
/// Tier 0 contains low level native words required for extending the language
pub fn scope(state: &mut State) -> Result<()> {
    state.new_mod("scope".to_string())?;

    state.add_native_word("push_frame", "(n -- )", |state| {
        let n = state.pop_i32()? as usize;
        //state.frames.resize(n, Object::None);
        state.frames.push(vec![Object::None; n]);
        Ok(())
    });

    state.add_native_word("pop_frame", "(n -- )", |state| {
        let n = state.pop_i32()? as usize;
        //state.frames.truncate(state.frames.len() - n);
        assert_eq!(n, state.frames.pop().unwrap().len());
        Ok(())
    });

    state.add_native_word("store", "(x addr -- )", |state| {
        let addr = state.pop_i32()? as usize;
        //let addr = state.frames.len() - addr - 1;
        let x = state.pop()?;
        //state.frames[addr] = x.clone();
        state.frames.last_mut().unwrap()[addr] = x;
        Ok(())
    });

    state.add_native_word("fetch", "(addr -- x)", |state| {
        let addr = state.pop_i32()? as usize;
        //let addr = state.frames.len() - addr - 1;
        let x = state.frames.last_mut().unwrap()[addr].clone();
        state.push(x)?;
        Ok(())
    });

    let push_frame = state.current_module.lookup("push_frame").unwrap().clone();
    let pop_frame = state.current_module.lookup("pop_frame").unwrap().clone();
    let store = state.current_module.lookup("store").unwrap().clone();
    let fetch = state.current_module.lookup("fetch").unwrap().clone();

    state.add_native_parse_word("set", move |state| {
        let name = state.next_token().ok_or(ErrorKind::EndOfInput)?;

        let i = state.scopes.last_mut().unwrap().get_storage_location(&name) as i32;

        let instructions = state.top_mut()?.try_as_quotation_mut()?;
        instructions.ops.push(Opcode::push_i32(i));
        instructions.ops.push(Opcode::call_word(store.clone()));
        Ok(())
    });

    state.add_native_parse_word("get", move |state| {
        let name = state.next_token().ok_or(ErrorKind::EndOfInput)?;

        let i = state.scopes.last_mut().unwrap().get_storage_location(&name) as i32;

        let instructions = state.top_mut()?.try_as_quotation_mut()?;
        instructions.ops.push(Opcode::push_i32(i));
        instructions.ops.push(Opcode::call_word(fetch.clone()));
        Ok(())
    });

    state.add_native_parse_word("::", move |state| {
        // todo: parse stack effect from word definition and compare against derived stack effect?

        let name = state.next_token().ok_or(ErrorKind::EndOfInput)?;

        state.scopes.push(CompilerScope::new());

        state.begin_compile();

        if let Err(e) = state.parse_until(";") {
            state.pop().unwrap();
            state.scopes.pop().unwrap();
            return Err(e);
        }

        let scope = state.scopes.pop().unwrap();
        let n_vars = scope.len() as i32;

        let mut quot = ByteCode::new();
        quot.ops.push(Opcode::push_i32(n_vars));
        quot.ops.push(Opcode::call_word(push_frame.clone()));
        quot.ops.extend(
            Rc::try_unwrap(state.pop()?.try_into_rc_quotation()?)
                .or(Err(ErrorKind::OwnershipError))?
                .ops,
        );
        quot.ops.push(Opcode::push_i32(n_vars));
        quot.ops.push(Opcode::call_word(pop_frame.clone()));

        let se = quot
            .ops
            .iter()
            .map(Opcode::stack_effect)
            .collect::<Result<_>>()?;

        state.add_compound_word(name, se, Rc::new(quot));
        Ok(())
    });

    state.exit_mod().unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tier0;

    #[test]
    fn scoped_words() {
        let state = &mut State::new();
        tier0(state).unwrap();
        scope(state).unwrap();

        state.run("USE scope:").unwrap();

        state.run("123").unwrap(); // push sentinel value on stack

        state.run(":: dup   set x get x get x ;").unwrap();
        state.run(":: swap   set x set y get x get y ;").unwrap();
        state
            .run(":: over   set b set a get a get b get a ;")
            .unwrap();
        state
            .run(":: rot   set c set b set a get b get c get a  ;")
            .unwrap();
        state.run(":: drop   set x ;").unwrap();

        state.run("42 dup").unwrap();
        state.assert_pop(42);
        state.assert_pop(42);

        state.run("12 34 swap").unwrap();
        state.assert_pop(12);
        state.assert_pop(34);

        state.run("56 78 over").unwrap();
        state.assert_pop(56);
        state.assert_pop(78);
        state.assert_pop(56);

        state.run("\"a\" \"b\" \"c\" rot").unwrap();
        assert_eq!(state.pop_string().unwrap(), "a");
        assert_eq!(state.pop_string().unwrap(), "c");
        assert_eq!(state.pop_string().unwrap(), "b");

        state.run("0 drop").unwrap();
        state.assert_pop(123);
    }
}
