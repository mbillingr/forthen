use std::rc::Rc;

use forthen_core::CompilerScope;
use forthen_core::Object;
use forthen_core::StackEffect;
use forthen_core::State;
use forthen_core::{Opcode, Quotation};

/// Load language tier 0 into the dictionary
///
/// Tier 0 contains low level native words required for extending the language
pub fn tier0(state: &mut State) {
    state.add_native_parse_word(";", |_| panic!("Unexpected Delimiter"));
    state.add_native_parse_word("]", |_| panic!("Unexpected Delimiter"));

    state.add_native_parse_word("SYNTAX:", |state| {
        let name = state.next_token().expect("word name");
        state.begin_compile();
        state.parse_until(";");
        let obj = state.pop();
        state.add_compound_parse_word(name, obj.into_rc_quotation());
    });

    state.add_native_parse_word(":", |state| {
        // todo: parse stack effect from word definition and compare against derived stack effect?

        let name = state.next_token().expect("word name");

        state.begin_compile();
        state.parse_until(";");
        let quot = state.pop().into_rc_quotation();

        let mut se = StackEffect::new();
        for op in &quot.ops {
            se = se.chain(&op.stack_effect());
        }

        state.add_compound_word(name, se, quot);
    });

    state.add_native_parse_word("[", |state| {
        state.begin_compile();
        state.parse_until("]");
        let quot = state.pop().into_rc_quotation();

        let mut se = StackEffect::new();
        for op in &quot.ops {
            se = se.chain(&op.stack_effect());
        }

        let obj = Object::Quotation(quot, se);
        state
            .top_mut()
            .as_quotation_mut()
            .ops
            .push(Opcode::Push(obj));
    });

    state.add_native_word("push_frame", "(n -- )", |state| {
        let n = state.pop().try_into_i32().unwrap() as usize;
        state.frames.resize(n, Object::None);
    });

    state.add_native_word("pop_frame", "(n -- )", |state| {
        let n = state.pop().try_into_i32().unwrap() as usize;
        state.frames.truncate(state.frames.len() - n);
    });

    state.add_native_word("store", "(x addr -- )", |state| {
        let addr = state.pop().try_into_i32().unwrap() as usize;
        let addr = state.frames.len() - addr - 1;
        let x = state.pop();
        state.frames[addr] = x.clone();
    });

    state.add_native_word("fetch", "(addr -- x)", |state| {
        let addr = state.pop().try_into_i32().unwrap() as usize;
        let addr = state.frames.len() - addr - 1;
        let x = state.frames[addr].clone();
        state.push(x);
    });

    let push_frame = state.dictionary.lookup("push_frame").unwrap().clone();
    let pop_frame = state.dictionary.lookup("pop_frame").unwrap().clone();
    let store = state.dictionary.lookup("store").unwrap().clone();
    let fetch = state.dictionary.lookup("fetch").unwrap().clone();

    state.add_closure_parse_word("set", move |state| {
        let name = state.next_token().expect("variable name");

        let i = state.scopes.last_mut().unwrap().get_storage_location(&name) as i32;

        let instructions = state.top_mut().as_quotation_mut();
        instructions.ops.push(Opcode::push_i32(i));
        instructions.ops.push(Opcode::call_word(store.clone()));
    });

    state.add_closure_parse_word("get", move |state| {
        let name = state.next_token().expect("variable name");

        let i = state.scopes.last_mut().unwrap().get_storage_location(&name) as i32;

        let instructions = state.top_mut().as_quotation_mut();
        instructions.ops.push(Opcode::push_i32(i));
        instructions.ops.push(Opcode::call_word(fetch.clone()));
    });

    state.add_closure_parse_word("::", move |state| {
        // todo: parse stack effect from word definition and compare against derived stack effect?

        let name = state.next_token().expect("word name");

        state.scopes.push(CompilerScope::new());

        state.begin_compile();
        state.parse_until(";");

        let scope = state.scopes.pop().unwrap();
        let n_vars = scope.len() as i32;

        let mut quot = Quotation::new();
        quot.ops.push(Opcode::push_i32(n_vars));
        quot.ops.push(Opcode::call_word(push_frame.clone()));
        quot.ops
            .extend(Rc::try_unwrap(state.pop().into_rc_quotation()).unwrap().ops);
        quot.ops.push(Opcode::push_i32(n_vars));
        quot.ops.push(Opcode::call_word(pop_frame.clone()));

        let mut se = StackEffect::new();
        for op in &quot.ops {
            se = se.chain(&op.stack_effect());
        }

        state.add_compound_word(name, se, Rc::new(quot));
    });

    state.add_native_word("call", "(..a func(..a -- ..b) -- ..b)", |state| {
        let func = state.pop();
        func.invoke(state);
    });

    state.add_native_word(
        "if",
        "(..a ? true(..a -- ..b) false(..a -- ..b) -- ..b)",
        |state| {
            let else_branch = state.pop();
            let if_branch = state.pop();
            let cond = state.pop().try_into_bool().unwrap();
            if cond {
                if_branch.invoke(state);
            } else {
                else_branch.invoke(state);
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_words() {
        let state = &mut State::new();
        tier0(state);

        state.run("123"); // push sentinel value on stack
        state.run(": the-answer 42 ;"); // define new word
        assert_eq!(state.pop_i32(), Some(123)); // make sure the word definition has no effect on the stack
        state.run("the-answer"); // run the new word
        assert_eq!(state.pop_i32(), Some(42));

        state.run(": more-answers the-answer the-answer ;");
        state.run("more-answers");
        assert_eq!(state.pop_i32(), Some(42));
        assert_eq!(state.pop_i32(), Some(42));
    }

    #[test]
    fn new_parse_words() {
        let state = &mut State::new();
        tier0(state);

        state.add_native_word("-rot", "(a b c -- c a b)", |state| {
            let a = state.pop();
            let b = state.pop();
            let c = state.pop();
            state.push(a);
            state.push(c);
            state.push(b);
        });
        state.add_native_word(".s", "( -- )", |state| println!("{:?}", state.stack));

        state.run("123"); // push sentinel value on stack
        state.run("SYNTAX: the-answer 42 -rot ;"); // define new parse word that puts a number deep in the stack
        state.run(": nop the-answer ; .s"); // define a new word
        assert_eq!(state.pop_i32(), Some(42)); // the number should end up on the stack during word definition
        state.run("nop"); // make sure the new word does nothing
        assert_eq!(state.pop_i32(), Some(123));
    }

    #[test]
    fn scoped_words() {
        let state = &mut State::new();
        tier0(state);

        state.run("123"); // push sentinel value on stack

        state.run(":: dup   set x get x get x ;");
        state.run(":: swap   set x set y get x get y ;");
        state.run(":: over   set b set a get a get b get a ;");
        state.run(":: rot   set c set b set a get b get c get a  ;");
        state.run(":: drop   set x ;");

        state.run("42 dup");
        assert_eq!(state.pop_i32(), Some(42));
        assert_eq!(state.pop_i32(), Some(42));

        state.run("12 34 swap");
        assert_eq!(state.pop_i32(), Some(12));
        assert_eq!(state.pop_i32(), Some(34));

        state.run("56 78 over");
        assert_eq!(state.pop_i32(), Some(56));
        assert_eq!(state.pop_i32(), Some(78));
        assert_eq!(state.pop_i32(), Some(56));

        state.run("\"a\" \"b\" \"c\" rot");
        assert_eq!(state.pop_str().unwrap(), "a");
        assert_eq!(state.pop_str().unwrap(), "c");
        assert_eq!(state.pop_str().unwrap(), "b");

        state.run("0 drop");
        assert_eq!(state.pop_i32(), Some(123));
    }

    #[test]
    fn quotations() {
        let state = &mut State::new();
        tier0(state);

        state.run("123"); // push sentinel value on stack

        state.run("[ 42 ]");
        assert_eq!(state.pop_i32(), None);

        state.run("[ 42 ] call");
        assert_eq!(state.pop_i32(), Some(42));

        assert_eq!(state.pop_i32(), Some(123));
    }

    #[test]
    fn if_word() {
        let state = &mut State::new();
        tier0(state);

        state.add_native_word("true", "( -- b)", |state| state.push(Object::True));
        state.add_native_word("false", "( -- b)", |state| state.push(Object::False));

        state.run(":: dup   set x get x get x ;");
        state.run(":: swap   set x set y get x get y ;");
        state.run(":: over   set b set a get a get b get a ;");
        state.run(":: rot   set c set b set a get b get c get a  ;");
        state.run(":: drop   set x ;");

        state.run("123"); // push sentinel value on stack

        state.run("false [ \"yes\" ] [ \"no\" ] if");
        assert_eq!(state.pop_str().unwrap(), "no");

        state.run("true [ \"yes\" ] [ \"no\" ] if");
        assert_eq!(state.pop_str().unwrap(), "yes");

        state.run(": yes-or-no [ \"yes\" dup ] [ \"no\" \"no\" ] if ;");

        state.run("false yes-or-no");
        assert_eq!(state.pop_str().unwrap(), "no");
        assert_eq!(state.pop_str().unwrap(), "no");

        state.run("true yes-or-no");
        assert_eq!(state.pop_str().unwrap(), "yes");
        assert_eq!(state.pop_str().unwrap(), "yes");

        assert_eq!(state.pop_i32(), Some(123));
    }
}
