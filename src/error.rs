#[derive(Debug)]
pub enum Error {
    // manipulation errors
    InvalidOperation(String),
    InvalidComment(String),
    NodeError(indextree::NodeError),

    // serializer
    NoPrefixForNamespace(String),

    // parser errors
    UnclosedTag,
    InvalidCloseTag(String, String),
    UnclosedEntity(String),
    InvalidEntity(String),
    UnknownPrefix(String),
    DuplicateAttribute(String),
    UnsupportedVersion(String),
    UnsupportedEncoding(String),
    UnsupportedNotStandalone,
    DtdUnsupported,
    Parser(xmlparser::Error),

    // io
    Io(std::io::Error),
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
