use std::error::Error as StdError;
use std::fmt;
use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    ParseError(ParseError),
    StackEffectError(StackEffectError),
    StackError(StackError),
    TypeError(TypeError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ParseError(e) => e.fmt(f),
            Error::StackEffectError(e) => e.fmt(f),
            Error::StackError(e) => e.fmt(f),
            Error::TypeError(e) => e.fmt(f),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            Error::ParseError(_) => "something went wrong during parsing",
            Error::StackEffectError(_) => "something went wrong with a stack effect",
            Error::StackError(_) => "something went wrong with the stack",
            Error::TypeError(_) => "something went wrong in the type system",
        }
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Error::ParseError(err)
    }
}

impl From<StackEffectError> for Error {
    fn from(err: StackEffectError) -> Self {
        Error::StackEffectError(err)
    }
}

impl From<ParseError> for Result<()> {
    fn from(err: ParseError) -> Self {
        Err(err.into())
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    EndOfInput,
    UnexpectedDelimiter(&'static str),
    GeneralExpectation(&'static str),
    UnknownWord(String),
    AmbiguousWord(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::UnexpectedDelimiter(s) => write!(f, "unexpected delimiter: {}", s),
            ParseError::GeneralExpectation(s) => write!(f, "expected {}", s),
            ParseError::UnknownWord(s) => write!(f, "unknown word: {}", s),
            ParseError::AmbiguousWord(s) => write!(f, "ambiguous word: {}", s),
            _ => write!(f, "{}", self.description()),
        }
    }
}

impl StdError for ParseError {
    fn description(&self) -> &str {
        match self {
            ParseError::EndOfInput => "unexpected end of input",
            ParseError::UnexpectedDelimiter(_) => "found unexpected delimiter",
            ParseError::GeneralExpectation(_) => "expected something else",
            ParseError::UnknownWord(_) => "unknown word",
            ParseError::AmbiguousWord(_) => "ambiguous word",
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum StackEffectError {
    ParseError(ParseError),
    Incompatible,
}

impl fmt::Display for StackEffectError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl StdError for StackEffectError {
    fn description(&self) -> &str {
        match self {
            StackEffectError::ParseError(_) => "error during stack effect parsing",
            StackEffectError::Incompatible => "attempted to combine incompatible stack effects",
        }
    }
}

impl From<ParseError> for StackEffectError {
    fn from(err: ParseError) -> Self {
        StackEffectError::ParseError(err)
    }
}

#[derive(Debug, PartialEq)]
pub enum TypeError {
    General(String),
    ExpectationError(String, String),
    ConversionError(String, String),
    RcUnwrapError,
    MutationError,
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TypeError::General(msg) => write!(f, "type error: {}", msg),
            TypeError::ExpectationError(a, b) => write!(f, "expected type {} but got {}", b, a),
            TypeError::ConversionError(a, b) => write!(f, "cannot convert {} to {}", a, b),
            _ => write!(f, "{}", self.description()),
        }
    }
}

impl StdError for TypeError {
    fn description(&self) -> &str {
        match self {
            TypeError::General(_) => "general type error",
            TypeError::ExpectationError(_, _) => "wrong type",
            TypeError::ConversionError(_, _) => "cannot convert type",
            TypeError::RcUnwrapError => "cannot unwrap Rc (likely there are other references)",
            TypeError::MutationError => "cannot mutate value",
        }
    }
}

impl From<TypeError> for Error {
    fn from(err: TypeError) -> Self {
        Error::TypeError(err)
    }
}

impl From<TypeError> for Result<()> {
    fn from(err: TypeError) -> Self {
        Err(err.into())
    }
}

#[derive(Debug, PartialEq)]
pub enum StackError {
    StackUnderflow,
}

impl fmt::Display for StackError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            _ => write!(f, "{}", self.description()),
        }
    }
}

impl StdError for StackError {
    fn description(&self) -> &str {
        match self {
            StackError::StackUnderflow => "stack underflow",
        }
    }
}

impl From<StackError> for Error {
    fn from(err: StackError) -> Self {
        Error::StackError(err)
    }
}

impl From<StackError> for Result<()> {
    fn from(err: StackError) -> Self {
        Err(err.into())
    }
}
