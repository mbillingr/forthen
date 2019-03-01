use forthen_core::State;
use forthen_std::{tier0, tier1};

fn main() {
    let state = &mut State::new();
    tier0(state);
    tier1(state);

    state.run("3 5 \"hello forth!\" .s");
    state.run("3 5 \"hello forth!\" .s");

    state.run(": the-answer 42 ;");
    state.run("the-answer .s");

    state.run(": more-answers the-answer the-answer ;");
    state.run(": 2dup over over ;");
    state.run(": stackfun swap 2dup swap ;");

    println!("{:#?}", state);

    state.print_dictionary();
}
