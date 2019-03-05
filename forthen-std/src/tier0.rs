use forthen_core::errors::*;
use forthen_core::CompilerScope;
use forthen_core::Object;
use forthen_core::StackEffect;
use forthen_core::State;
use forthen_core::{Opcode, Quotation};
use std::rc::Rc;

/// Load language tier 0 into the dictionary
///
/// Tier 0 contains low level native words required for extending the language
pub fn tier0(state: &mut State) -> Result<()> {
    state.add_native_parse_word(";", |_| Err(ErrorKind::UnexpectedDelimiter(";").into()));
    state.add_native_parse_word("]", |_| Err(ErrorKind::UnexpectedDelimiter("]").into()));

    state.add_native_parse_word("SYNTAX:", |state| {
        let name = state.next_token().expect("word name");
        state.begin_compile();

        if let Err(e) = state.parse_until(";") {
            state.pop().unwrap();
            return Err(e);
        }

        let obj = state.pop()?;
        state.add_compound_parse_word(name, obj.try_into_rc_quotation()?);
        Ok(())
    });

    state.add_native_parse_word(":", |state| {
        // todo: parse stack effect from word definition and compare against derived stack effect?

        let name = state.next_token().expect("word name");

        state.begin_compile();

        if let Err(e) = state.parse_until(";") {
            state.pop().unwrap();
            return Err(e);
        }

        let quot = state.pop()?.try_into_rc_quotation()?;

        let mut se = StackEffect::new();
        for op in &quot.ops {
            se = se.chain(&op.stack_effect()?)?;
        }

        state.add_compound_word(name, se, quot);
        Ok(())
    });

    state.add_native_parse_word("[", |state| {
        state.begin_compile();

        if let Err(e) = state.parse_until("]") {
            state.pop().unwrap();
            return Err(e);
        }
        let quot = state.pop()?.try_into_rc_quotation()?;

        let mut se = StackEffect::new();
        for op in &quot.ops {
            se = se.chain(&op.stack_effect()?)?;
        }

        let obj = Object::Quotation(quot, se);
        state
            .top_mut()?
            .try_as_quotation_mut()?
            .ops
            .push(Opcode::Push(obj));
        Ok(())
    });

    state.add_native_word("push_frame", "(n -- )", |state| {
        let n = state.pop_i32()? as usize;
        state.frames.resize(n, Object::None);
        Ok(())
    });

    state.add_native_word("pop_frame", "(n -- )", |state| {
        let n = state.pop_i32()? as usize;
        state.frames.truncate(state.frames.len() - n);
        Ok(())
    });

    state.add_native_word("store", "(x addr -- )", |state| {
        let addr = state.pop_i32()? as usize;
        let addr = state.frames.len() - addr - 1;
        let x = state.pop()?;
        state.frames[addr] = x.clone();
        Ok(())
    });

    state.add_native_word("fetch", "(addr -- x)", |state| {
        let addr = state.pop_i32()? as usize;
        let addr = state.frames.len() - addr - 1;
        let x = state.frames[addr].clone();
        state.push(x)?;
        Ok(())
    });

    let push_frame = state.dictionary.lookup("push_frame").unwrap().clone();
    let pop_frame = state.dictionary.lookup("pop_frame").unwrap().clone();
    let store = state.dictionary.lookup("store").unwrap().clone();
    let fetch = state.dictionary.lookup("fetch").unwrap().clone();

    state.add_closure_parse_word("set", move |state| {
        let name = state.next_token().ok_or(ErrorKind::EndOfInput)?;

        let i = state.scopes.last_mut().unwrap().get_storage_location(&name) as i32;

        let instructions = state.top_mut()?.try_as_quotation_mut()?;
        instructions.ops.push(Opcode::push_i32(i));
        instructions.ops.push(Opcode::call_word(store.clone()));
        Ok(())
    });

    state.add_closure_parse_word("get", move |state| {
        let name = state.next_token().ok_or(ErrorKind::EndOfInput)?;

        let i = state.scopes.last_mut().unwrap().get_storage_location(&name) as i32;

        let instructions = state.top_mut()?.try_as_quotation_mut()?;
        instructions.ops.push(Opcode::push_i32(i));
        instructions.ops.push(Opcode::call_word(fetch.clone()));
        Ok(())
    });

    state.add_closure_parse_word("::", move |state| {
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

        let mut quot = Quotation::new();
        quot.ops.push(Opcode::push_i32(n_vars));
        quot.ops.push(Opcode::call_word(push_frame.clone()));
        quot.ops.extend(
            Rc::try_unwrap(state.pop()?.try_into_rc_quotation()?)
                .or(Err(ErrorKind::OwnershipError))?
                .ops,
        );
        quot.ops.push(Opcode::push_i32(n_vars));
        quot.ops.push(Opcode::call_word(pop_frame.clone()));

        let mut se = StackEffect::new();
        for op in &quot.ops {
            se = se.chain(&op.stack_effect()?)?;
        }

        state.add_compound_word(name, se, Rc::new(quot));
        Ok(())
    });

    state.add_native_word("call", "(..a func(..a -- ..b) -- ..b)", |state| {
        let func = state.pop()?;
        func.invoke(state)
    });

    state.add_native_word(
        "if",
        "(..a ? true(..a -- ..b) false(..a -- ..b) -- ..b)",
        |state| {
            let else_branch = state.pop()?;
            let if_branch = state.pop()?;
            let cond = state.pop()?.try_into_bool()?;
            if cond {
                if_branch.invoke(state)
            } else {
                else_branch.invoke(state)
            }
        },
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_words() {
        let state = &mut State::new();
        tier0(state).unwrap();

        state.run("123").unwrap(); // push sentinel value on stack
        state.run(": the-answer 42 ;").unwrap(); // define new word
        assert_eq!(state.pop_i32().unwrap(), 123); // make sure the word definition has no effect on the stack
        state.run("the-answer").unwrap(); // run the new word
        assert_eq!(state.pop_i32().unwrap(), 42);

        state.run(": more-answers the-answer the-answer ;").unwrap();
        state.run("more-answers").unwrap();
        assert_eq!(state.pop_i32().unwrap(), 42);
        assert_eq!(state.pop_i32().unwrap(), 42);
    }

    #[test]
    fn new_parse_words() {
        let state = &mut State::new();
        tier0(state).unwrap();

        state.add_native_word("-rot", "(a b c -- c a b)", |state| {
            let a = state.pop()?;
            let b = state.pop()?;
            let c = state.pop()?;
            state.push(a)?;
            state.push(c)?;
            state.push(b)?;
            Ok(())
        });
        state.add_native_word(".s", "( -- )", |state| {
            println!("{:?}", state.stack);
            Ok(())
        });

        state.run("123").unwrap(); // push sentinel value on stack
        state.run("SYNTAX: the-answer 42 -rot ;").unwrap(); // define new parse word that puts a number deep in the stack
        state.run(": nop the-answer ; .s").unwrap(); // define a new word
        assert_eq!(state.pop_i32().unwrap(), 42); // the number should end up on the stack during word definition
        state.run("nop").unwrap(); // make sure the new word does nothing
        assert_eq!(state.pop_i32().unwrap(), 123);
    }

    #[test]
    fn scoped_words() {
        let state = &mut State::new();
        tier0(state).unwrap();

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
        assert_eq!(state.pop_str().unwrap(), "a");
        assert_eq!(state.pop_str().unwrap(), "c");
        assert_eq!(state.pop_str().unwrap(), "b");

        state.run("0 drop").unwrap();
        state.assert_pop(123);
    }

    #[test]
    fn quotations() {
        let state = &mut State::new();
        tier0(state).unwrap();;

        state.run("123").unwrap();; // push sentinel value on stack

        state.run("[ 42 ]").unwrap();;
        assert!(state.pop_i32().is_err());

        state.run("[ 42 ] call").unwrap();;
        state.assert_pop(42);

        state.assert_pop(123);
    }

    #[test]
    fn if_word() {
        let state = &mut State::new();
        tier0(state).unwrap();

        state.add_native_word("true", "( -- b)", |state| state.push(Object::True));
        state.add_native_word("false", "( -- b)", |state| state.push(Object::False));

        state.run(":: dup   set x get x get x ;").unwrap();
        state.run(":: swap   set x set y get x get y ;").unwrap();
        state
            .run(":: over   set b set a get a get b get a ;")
            .unwrap();
        state
            .run(":: rot   set c set b set a get b get c get a  ;")
            .unwrap();
        state.run(":: drop   set x ;").unwrap();

        state.run("123").unwrap(); // push sentinel value on stack

        state.run("false [ \"yes\" ] [ \"no\" ] if").unwrap();
        assert_eq!(state.pop_str().unwrap(), "no");

        state.run("true [ \"yes\" ] [ \"no\" ] if").unwrap();
        assert_eq!(state.pop_str().unwrap(), "yes");

        state
            .run(": yes-or-no [ \"yes\" dup ] [ \"no\" \"no\" ] if ;")
            .unwrap();

        state.run("false yes-or-no").unwrap();
        assert_eq!(state.pop_str().unwrap(), "no");
        assert_eq!(state.pop_str().unwrap(), "no");

        state.run("true yes-or-no").unwrap();
        assert_eq!(state.pop_str().unwrap(), "yes");
        assert_eq!(state.pop_str().unwrap(), "yes");

        state.assert_pop(123);
    }
}
