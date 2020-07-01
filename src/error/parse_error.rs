pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub enum ParseError {}
