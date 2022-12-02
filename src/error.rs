use crate::namespace::Namespace;

#[derive(Debug)]
pub enum Error {
    NoPrefixForNamespace(String),
    UnknownPrefix(String),
    Io(std::io::Error),
    NodeId(id_tree::NodeIdError),
    Parser(xmlparser::Error),
}

impl From<xmlparser::Error> for Error {
    #[inline]
    fn from(e: xmlparser::Error) -> Self {
        Error::Parser(e)
    }
}

impl From<id_tree::NodeIdError> for Error {
    #[inline]
    fn from(e: id_tree::NodeIdError) -> Self {
        Error::NodeId(e)
    }
}

impl From<std::io::Error> for Error {
    #[inline]
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}
