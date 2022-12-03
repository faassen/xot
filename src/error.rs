#[derive(Debug)]
pub enum Error {
    UnclosedEntity(String),
    InvalidEntity(String),
    NoPrefixForNamespace(String),
    UnknownPrefix(String),
    Io(std::io::Error),
    Parser(xmlparser::Error),
}

impl From<xmlparser::Error> for Error {
    #[inline]
    fn from(e: xmlparser::Error) -> Self {
        Error::Parser(e)
    }
}

impl From<std::io::Error> for Error {
    #[inline]
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}
