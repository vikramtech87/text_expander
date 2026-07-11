use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum CodecError {
    UnexpectedEof,
    InvalidUtf8,
    InvalidTypeTag(u8),
}

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CodecError::UnexpectedEof => write!(f, "Unexpected EOF"),
            CodecError::InvalidUtf8 => write!(f, "Invalid UTF-8"),
            CodecError::InvalidTypeTag(tag) => write!(f, "Invalid type tag: {}", tag),
        }
    }
}

#[derive(Debug)]
pub enum ParseExpansionError {
    MalformedPlaceholder(String),
}