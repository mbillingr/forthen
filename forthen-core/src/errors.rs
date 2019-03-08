error_chain! {
    links {
    }

    errors {
        // parsing errors
        EndOfInput
        UnexpectedDelimiter(t: &'static str)

        // stack effect errors
        IncompatibleStackEffects

        // language errors
        AmbiguousWord(word: String)
        UnknownWord(word: String)
        StackUnderflow

        // type system errors
        TypeError(t: String)
        OwnershipError

        AttributeError(t: String)
    }
}
