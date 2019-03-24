error_chain! {
    links {
    }

    errors {
        // parsing errors
        EndOfInput
        UnexpectedDelimiter(t: &'static str) {
            display("Unexpected Delimiter: {:?}", t)
        }
        ExpectedStackEffect
        PathError

        // stack effect errors
        IncompatibleStackEffects
        InfiniteSubstitution

        // language errors
        AmbiguousWord(word: String) {
            display("Ambiguous Word: {}", word)
        }
        UnknownWord(word: String) {
            display("Unkown Word: {}", word)
        }
        StackUnderflow

        // type system errors
        TypeError(t: String) {
            display("Type Error: {}", t)
        }
        OwnershipError

        AttributeError(t: String) {
            display("Attribute Error: {}", t)
        }

        IndexError(i: usize, l: usize) {
            display("Index Error: {} but length is {}", i, l)
        }

        RuntimeError(msg: String) {
            display("Runtime Error: {}", msg)
        }
    }
}
