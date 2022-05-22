use crate::Rule;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum XTagError {
    #[error("io error")]
    File(#[from] ::std::io::Error), // FIXME check

    #[error("utf8 error")]
    Charset(#[from] ::std::str::Utf8Error), // FIXME check

    #[error("parse error")]
    Parser(#[from] ::pest::error::Error<Rule>),

    // Used when grammar and parser implementation are incompatible
    #[error("parser implementation error {0}")]
    ParserImplementation(String),

    #[error("regex error")]
    Regex(#[from] ::regex::Error), // FIXME check

    #[error("int parser error")]
    IntParse(#[from] ::core::num::ParseIntError),
}

pub type Result<T> = std::result::Result<T, XTagError>;
