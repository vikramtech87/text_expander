#[derive(Debug)]
pub enum CodecError {
    UnexpectedEof,
    InvalidUtf8,
    InvalidTypeTag(u8),
}

#[derive(Debug)]
pub enum ParseExpansionError {
    MalformedPlaceholder(String),
}