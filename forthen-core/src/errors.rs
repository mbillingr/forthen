error_chain! {
    links {
    }

    errors {
        // parsing errors
        EndOfInput
        UnexpectedDelimiter(t: &'static str) {
            display("Unexpected Delimiter: {:?}", t)
        }
        PathError
        ExpectedStackEffect

        // stack effect errors
        IncompatibleStackEffects

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
    }
}
