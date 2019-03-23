use error_chain::ChainedError;
use forthen_core::errors::*;
use forthen_core::objects::prelude::*;
use forthen_core::State;
use forthen_std::*;
use rustyline::Editor;
use std::env;
use std::fs;

fn main() {
    let mut state = State::new();
    stdlib(&mut state).unwrap();

    /*state.add_native_word("std:tier0", "( -- )", |state| tier0(state));
    state.add_native_word("std:tier1", "( -- )", |state| tier1(state));
    state.add_native_word("std:ops", "( -- )", |state| ops(state));
    state.add_native_word("std:complex", "( -- )", |state| complex(state));*/
    state.add_native_word("words", "( -- )", |state| {
        state.print_dictionary();
        Ok(())
    });

    let args: Vec<String> = env::args().collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();

    let repl;
    let file;

    match &args[1..] {
        ["-i", cmd] => {
            repl = true;
            file = Some(*cmd);
        }
        [cmd] => {
            repl = false;
            file = Some(*cmd);
        }
        [] => {
            repl = true;
            file = None;
        }
        _ => {
            eprintln!("Invalid Arguments. Expected script, -i script, or nothing.");
            return;
        }
    }

    if let Some(filename) = file {
        let code =
            fs::read_to_string(filename).unwrap_or_else(|_| panic!("Unable to load {}", filename));

        match state.run(&code) {
            Ok(()) => {}
            Err(e) => report_error(e),
        }
    }

    if !repl {
        return;
    }

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
    eprintln!("{}", e);
    eprintln!("{}", e.display_chain().to_string());
}
