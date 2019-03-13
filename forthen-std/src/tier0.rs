use forthen_core::errors::*;
use forthen_core::object_factory::StringManager;
use forthen_core::objects::prelude::*;
use forthen_core::Object;
use forthen_core::Opcode;
use forthen_core::State;

/// Load language tier 0 into the dictionary
///
/// Tier 0 contains low level native words required for extending the language
pub fn tier0(state: &mut State) -> Result<()> {
    state.add_native_parse_word(";", |_| Err(ErrorKind::UnexpectedDelimiter(";").into()));
    state.add_native_parse_word("]", |_| Err(ErrorKind::UnexpectedDelimiter("]").into()));

    state.add_native_word("next_token", "( -- token)", |state| {
        let token = state.next_token().expect("token");
        let token = state.factory.get_string(token);
        state.push(token)?;
        Ok(())
    });

    state.add_native_parse_word("MODULE", |state| {
        let name = state.next_token().ok_or(ErrorKind::EndOfInput)?;
        state.new_mod(name)
    });

    state.add_native_parse_word("USE", |state| {
        let fullpath = state.next_token().ok_or(ErrorKind::EndOfInput)?;

        let mut split = fullpath.rsplitn(2, ':');
        let word = split.next().ok_or(ErrorKind::PathError)?;
        let mut path = split.next().ok_or(ErrorKind::PathError)?;

        let relative = if path.starts_with(':') {
            path = &path[1..];
            state.root_module()
        } else {
            &state.current_module
        };

        let target_mod = relative.access_path(path).ok_or(ErrorKind::PathError)?;

        if word != "" {
            let word_id = target_mod
                .lookup(word)
                .ok_or_else(|| ErrorKind::UnknownWord(fullpath))?;

            state
                .current_module
                .insert_ref(word_id.name.clone(), word_id);
        } else {
            for name in target_mod.local_keys() {
                let word_id = target_mod
                    .lookup(&*name)
                    .ok_or_else(|| ErrorKind::UnknownWord(fullpath.clone()))?;

                state
                    .current_module
                    .insert_ref(word_id.name.clone(), word_id);
            }
        }

        Ok(())
    });

    state.add_native_parse_word("SYNTAX:", |state| {
        let name = state.next_token().ok_or(ErrorKind::EndOfInput)?;
        state.begin_compile();

        if let Err(e) = state.parse_until(";") {
            state.pop().unwrap();
            return Err(e);
        }

        let obj = state.pop()?;
        state.add_compound_parse_word(name, obj.try_into_rc_quotation()?);
        Ok(())
    });

    state.add_native_parse_word("LET:", |state| {
        let name = state.next_token().ok_or(ErrorKind::EndOfInput)?;

        state.begin_compile();

        if let Err(e) = state.parse_until(";") {
            state.pop().unwrap();
            return Err(e);
        }

        let obj = state.pop()?;
        obj.call(state)?;

        let value = state.pop()?;

        state.add_native_word(name, "( -- x)", move |state| state.push(value.clone()));
        Ok(())
    });

    state.add_native_parse_word(":", |state| {
        // todo: parse stack effect from word definition and compare against derived stack effect?

        let name = state.next_token().ok_or(ErrorKind::EndOfInput)?;

        state.begin_compile();

        if let Err(e) = state.parse_until(";") {
            state.pop().unwrap();
            return Err(e);
        }

        let quot = state.pop()?.try_into_rc_quotation()?;

        let se = quot
            .ops
            .iter()
            .map(Opcode::stack_effect)
            .collect::<Result<_>>()?;

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

        let se = quot
            .ops
            .iter()
            .map(Opcode::stack_effect)
            .collect::<Result<_>>()?;

        let obj = Object::Function(state.compile(quot, se));
        state
            .top_mut()?
            .try_as_quotation_mut()?
            .ops
            .push(Opcode::Push(obj));
        Ok(())
    });

    state.add_native_parse_word("(", |state| {
        let mut nesting = 1;
        while nesting > 0 {
            match state.next_token().as_ref().map(String::as_str) {
                None => return Err(ErrorKind::EndOfInput.into()),
                Some("(") => nesting += 1,
                Some(")") => nesting -= 1,
                Some(_) => {}
            }
        }
        Ok(())
    });

    state.add_native_word("call", "(..a func(..a -- ..b) -- ..b)", |state| {
        let func = state.pop()?;
        func.call(state)
    });

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

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scope;

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
        scope(state).unwrap();

        state.run("USE scope:").unwrap();

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
        assert_eq!(state.pop_string().unwrap(), "no");

        state.run("true [ \"yes\" ] [ \"no\" ] if").unwrap();
        assert_eq!(state.pop_string().unwrap(), "yes");

        state
            .run(": yes-or-no [ \"yes\" dup ] [ \"no\" \"no\" ] if ;")
            .unwrap();

        state.run("false yes-or-no").unwrap();
        assert_eq!(state.pop_string().unwrap(), "no");
        assert_eq!(state.pop_string().unwrap(), "no");

        state.run("true yes-or-no").unwrap();
        assert_eq!(state.pop_string().unwrap(), "yes");
        assert_eq!(state.pop_string().unwrap(), "yes");

        state.assert_pop(123);
    }
}
