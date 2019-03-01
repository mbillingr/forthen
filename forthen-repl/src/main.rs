use forthen_core::State;

fn main() {
    let mut state = State::new();
    state.tier0();
    state.tier1();

    state.run("3 5 \"hello forth!\" .s");
    state.run("3 5 \"hello forth!\" .s");

    state.run(": the-answer 42 ;");
    state.run("the-answer .s");

    state.run(": more-answers the-answer the-answer ;");
    state.run(": 2dup over over ;");
    state.run(": stackfun swap 2dup swap ;");

    println!("{:#?}", state);

    state.format_word(":");
    state.format_word("call");
    state.format_word("if");
    state.format_word("the-answer");
    state.format_word("more-answers");
    state.format_word("2dup");
    state.format_word("stackfun");
}
