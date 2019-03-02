use forthen_core::State;
use forthen_std::{ops, tier0, tier1};

use rustyline::Editor;

fn main() {
    let state = &mut State::new();
    tier0(state).unwrap();
    tier1(state).unwrap();

    state.run("3 5 \"hello forth!\" .s").unwrap();
    state.run("3 5 \"hello forth!\" .s").unwrap();

    state.run(": the-answer 42 ;").unwrap();
    state.run("the-answer .s").unwrap();

    state.run(": more-answers the-answer the-answer ;").unwrap();
    state.run(": 2dup over over ;").unwrap();
    state.run(": stackfun swap 2dup swap ;").unwrap();

    println!("{:#?}", state);

    state.print_dictionary();

    // --------------------

    let mut state = State::new();

    state.add_native_word("std:tier0", "( -- )", |state| tier0(state));
    state.add_native_word("std:tier1", "( -- )", |state| tier1(state));
    state.add_native_word("std:ops", "( -- )", |state| ops(state));
    state.add_native_word("words", "( -- )", |state| {
        state.print_dictionary();
        Ok(())
    });

    let mut rl = Editor::<()>::new();

    loop {
        let mut total_length = 0;
        let mut top = vec![];
        for x in state.stack.iter().rev() {
            let repr = format!("{:?}", x);
            total_length += repr.len() + 2;
            if total_length > 70 {
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

        match rl.readline(">> ") {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                match state.run(&line) {
                    Ok(()) => {}
                    Err(e) => println!("Error: {}", e),
                }
            }
            _ => {
                println!("Input Error");
                break;
            }
        }
    }
}
