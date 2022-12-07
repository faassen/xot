#[derive(Debug)]
pub enum Error {
    UnclosedTag,
    InvalidCloseTag(String, String),
    InvalidOperation(String),
    UnclosedEntity(String),
    InvalidEntity(String),
    NoPrefixForNamespace(String),
    UnknownPrefix(String),
    DuplicateAttribute(String),
    NodeError(indextree::NodeError),
    Io(std::io::Error),
    Parser(xmlparser::Error),
}

impl From<indextree::NodeError> for Error {
    #[inline]
    fn from(e: indextree::NodeError) -> Self {
        Error::NodeError(e)
    }
}

impl From<std::io::Error> for Error {
    #[inline]
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<xmlparser::Error> for Error {
    #[inline]
    fn from(e: xmlparser::Error) -> Self {
        Error::Parser(e)
    }
}
