use forthen_core::errors::*;
use forthen_core::objects::prelude::*;
use forthen_core::State;
use forthen_std::*;
use rustyline::Editor;

fn main() {
    let state = &mut State::new();
    tier0(state).unwrap();
    tier1(state).unwrap();

    //state.run("USE :tier0:").unwrap();
    state.run("USE :tier1:").unwrap();

    state.run("3 5 \"hello forth!\" .s").unwrap();
    state.run("3 5 \"hello forth!\" .s").unwrap();

    state.run(": the-answer 42 ;").unwrap();
    state.run("the-answer .s").unwrap();

    state.run(": more-answers the-answer the-answer ;").unwrap();
    state.run(": 2dup over over ;").unwrap();
    state.run(": stackfun swap 2dup swap ;").unwrap();

    println!("{:#?}", state);

    // --------------------

    let mut state = State::new();
    tier0(&mut state).unwrap();
    scope(&mut state).unwrap();
    tier1(&mut state).unwrap();
    ops(&mut state).unwrap();
    class(&mut state).unwrap();

    /*state.add_native_word("std:tier0", "( -- )", |state| tier0(state));
    state.add_native_word("std:tier1", "( -- )", |state| tier1(state));
    state.add_native_word("std:ops", "( -- )", |state| ops(state));
    state.add_native_word("std:complex", "( -- )", |state| complex(state));*/
    state.add_native_word("words", "( -- )", |state| {
        state.print_dictionary();
        Ok(())
    });

    state.print_dictionary();

    let mut rl = Editor::<()>::new();

    loop {
        println!();
        print_stack(&mut state, 70);

        match rl.readline(">> ") {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                match state.run(&line) {
                    Ok(()) => {}
                    Err(e) => report_error(e),
                }
            }
            _ => {
                println!("Input Error");
                break;
            }
        }
    }
}

fn print_stack(state: &mut State, max_len: usize) {
    let mut total_length = 0;
    let mut top = vec![];

    let stack_copy: Vec<_> = state
        .stack
        .iter()
        .rev()
        .cloned()
        .take(max_len / 3)
        .collect();

    for x in stack_copy {
        x.repr(state).unwrap();
        let repr = state.pop().unwrap().as_str().unwrap().to_string();
        total_length += repr.len() + 2;
        if total_length > max_len {
            break;
        }
        top.push(repr);
    }

    top.reverse();

    if top.len() < state.stack.len() {
        println!("[.., {}]", top.join(", "));
    } else {
        println!("[{}]", top.join(", "));
    }
}

fn report_error(e: Error) {
    eprintln!("{}", e)
}
