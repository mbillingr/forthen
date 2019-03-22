use forthen_core::errors::*;
use forthen_core::object_factory::StringManager;
use forthen_core::objects::prelude::*;
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
        state.add_compound_parse_word(name, obj);
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

        let mut se = state.next_token().ok_or(ErrorKind::EndOfInput)?;
        if se != "(" {
            return Err(ErrorKind::ExpectedStackEffect.into());
        }
        loop {
            let token = state.next_token().ok_or(ErrorKind::EndOfInput)?;
            se += " ";
            se += &token;
            if token == ")" {
                break;
            }
        }

        state.begin_compile();

        if let Err(e) = state.parse_until(";") {
            state.pop().unwrap();
            return Err(e);
        }

        let quot = state.pop()?;

        state.add_compound_word(name, se, quot);
        Ok(())
    });

    state.add_native_parse_word("[", |state| {
        state.begin_compile();

        if let Err(e) = state.parse_until("]") {
            state.pop().unwrap();
            return Err(e);
        }

        let code = state.pop()?;

        state.top_mut()?.as_vec_mut()?.push(code);
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
        state.run(": the-answer ( -- x ) 42 ;").unwrap(); // define new word
        assert_eq!(state.pop_i32().unwrap(), 123); // make sure the word definition has no effect on the stack
        state.run("the-answer").unwrap(); // run the new word
        assert_eq!(state.pop_i32().unwrap(), 42);

        state
            .run(": more-answers ( -- x x ) the-answer the-answer ;")
            .unwrap();
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
        state.run(": nop ( -- ) the-answer ; .s").unwrap(); // define a new word
        assert_eq!(state.pop_i32().unwrap(), 42); // the number should end up on the stack during word definition
        state.run("nop").unwrap(); // make sure the new word does nothing
        assert_eq!(state.pop_i32().unwrap(), 123);
    }

    #[test]
    fn modules() {
        let state = &mut State::new();
        tier0(state).unwrap();

        state.run(": in_root ( -- x ) 1 ;").unwrap();

        state.run("MODULE outer").unwrap();
        state.run("    : in_outer ( -- x ) 2 ;").unwrap();
        state
            .run("    : use_same_module ( -- x ) in_outer ;")
            .unwrap();
        state
            .run("    : use_parent_module ( -- x ) in_root ;")
            .unwrap();

        // make sure we can run words defined in the current module
        state.assert_run_pop("in_outer", &[2]);
        state.assert_run_pop("use_same_module", &[2]);
        state.assert_run_pop("use_parent_module", &[1]);
        // make sure we can run words defined in outer modules
        state.assert_run_pop("in_root", &[1]);

        state.run("    MODULE inner").unwrap();
        state.run("        : in_inner ( -- x ) 3 ;").unwrap();
        state
            .run("        : use_same_module ( -- x ) in_inner ;")
            .unwrap();
        state
            .run("        : use_parent_module ( -- x ) in_outer ;")
            .unwrap();
        state
            .run("        : use_root_module ( -- x ) in_root ;")
            .unwrap();

        // make sure we can run words defined in the current module
        state.assert_run_pop("in_inner", &[3]);
        // make sure words in the inner module take precedence over words with the same name in higher modules
        state.assert_run_pop("use_same_module", &[3]);
        state.assert_run_pop("use_parent_module", &[2]);
        state.assert_run_pop("use_root_module", &[1]);
        // make sure we can run words in outer modules
        state.assert_run_pop("in_outer", &[2]);
        state.assert_run_pop("in_root", &[1]);

        // use word from nested submodule given absolute path
        state.run("USE outer:inner:in_inner").is_err();
        state.run("USE :outer:inner:in_inner").unwrap();
        state.assert_run_pop("in_inner", &[3]);

        state.run("    END-MODULE").unwrap();

        // make sure we can't access words inside submodules
        assert!(state.run("in_inner").is_err());

        // make sure words in the current module were not overwritten by the submodule
        state.assert_run_pop("in_outer", &[2]);
        state.assert_run_pop("use_same_module", &[2]);
        state.assert_run_pop("use_parent_module", &[1]);

        state.run("END-MODULE").unwrap();

        // make sure we can't access words inside submodules
        assert!(state.run("in_outer").is_err());
        assert!(state.run("in_inner").is_err());

        // use word from submodule
        state.run("USE outer:in_outer").unwrap();
        state.assert_run_pop("in_outer", &[2]);

        // we should not be able to use a word from a nested submodule
        state.run("USE outer:in_inner").is_err();
        assert!(state.run("in_inner").is_err());

        // use word from nested submodule
        state.run("USE outer:inner:in_inner").unwrap();
        state.assert_run_pop("in_inner", &[3]);

        // use all words from nested submodule does not import from parent
        state.run("USE outer:inner:").unwrap();
        state.assert_run_pop("use_same_module", &[3]);
        state.assert_run_pop("use_parent_module", &[2]);
        state.assert_run_pop("use_root_module", &[1]);

        // use all words from other submodule overwrites definitions
        state.run("USE outer:").unwrap();
        state.assert_run_pop("use_same_module", &[2]);
        state.assert_run_pop("use_parent_module", &[1]);
    }

    #[test]
    fn quotations() {
        let state = &mut State::new();
        tier0(state).unwrap();

        state.run("123").unwrap();; // push sentinel value on stack

        state.run("[ 42 ]").unwrap();;
        assert!(state.pop_i32().is_err());

        state.run("[ 42 ] call").unwrap();;
        state.assert_pop(42);

        state.assert_pop(123);
    }
}
