use crate::xotdata::Node;

/// Xot errors
#[derive(Debug)]
pub enum Error {
    // access errors
    /// The node is not a Document node.
    NotDocument(Node),

    // manipulation errors
    /// Invalid operation on XML. You get this when
    /// trying to remove the document, or trying to
    /// insert something under a text node, for instance.
    InvalidOperation(String),

    /// You aren't allowed to use this string as a comment.
    /// Happens if you include `--` in a comment.
    InvalidComment(String),
    /// You aren't allowed to use this string as a processing instruction
    /// target. Happens if you use `XML` or any case variation of this.
    InvalidTarget(String),
    /// The node you tried to act on is not an element.
    NotElement(Node),
    /// Indextree error that can happen during manipulation.
    NodeError(indextree::NodeError),

    // serializer
    /// Missing prefix for namespace.
    /// Can occur during serialization if a namespace is used that has no
    /// prefix is declared. Use [`Xot::create_missing_prefixes`](crate::xotdata::Xot::create_missing_prefixes)
    /// to fix this.
    MissingPrefix(String),
    /// It's not allowed to serialize a processing instruction to HTML with a > in it.
    ProcessingInstructionGtInHtml(String),

    /// It's not allowed to include a namespace prefix in a processing instruction
    /// target name.
    NamespaceInProcessingInstruction,

    // parser errors
    /// The XML is not well-formed - a tag is opened and never closed.
    UnclosedTag,
    /// The XML is not well-formed - a tag is closed that was never opened.
    InvalidCloseTag(String, String),
    /// The XML is not well-formed - you use `&` to open an entity without
    /// closing it with `;`.
    UnclosedEntity(String),
    /// The entity is not known. Only the basic entities are supported
    /// right now, not any user defined ones.
    InvalidEntity(String),
    /// You used a namespace prefix that is not declared.
    UnknownPrefix(String),
    /// You declared an attribute of the same name twice.
    DuplicateAttribute(String),
    /// Unsupported XML version. Only 1.0 is supported.
    UnsupportedVersion(String),
    /// Unsupported XML encoding. Only UTF-8 is supported.
    UnsupportedEncoding(String),
    /// Unsupported standalone declaration. Only `yes` is supported.
    UnsupportedNotStandalone,
    /// XML DTD is not supported.
    DtdUnsupported,
    /// Illegal content that can never appear under a document node, such as an
    /// attribute or a namespace node
    IllegalAtTopLevel(Node),
    /// A text node at top level is not allowed in a well formed document,
    /// but it is accepted as a fragment.
    TextAtTopLevel(Node),
    /// Missing document element at top level
    NoElementAtTopLevel,
    /// Multiple document elements at top level
    MultipleElementsAtTopLevel,
    /// xmlparser error
    Parser(xmlparser::Error),
    /// IO error
    ///
    /// We take the string version of the IO error so as to keep errors comparable,
    /// which is more important than the exact error object in this case (serialization)
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

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::NotDocument(_) => write!(f, "Not a document node"),
            Error::InvalidOperation(s) => write!(f, "Invalid operation: {}", s),
            Error::InvalidComment(s) => write!(f, "Invalid comment: {}", s),
            Error::InvalidTarget(s) => write!(f, "Invalid target: {}", s),
            Error::NotElement(_) => write!(f, "Not an element"),
            Error::NodeError(e) => write!(f, "Node error: {}", e),
            Error::MissingPrefix(_) => write!(f, "Missing prefix"),
            Error::ProcessingInstructionGtInHtml(s) => {
                write!(f, "Processing instruction with > in HTML: {}", s)
            }
            Error::NamespaceInProcessingInstruction => {
                write!(f, "Namespace in processing instruction target")
            }
            Error::UnclosedTag => write!(f, "Unclosed tag"),
            Error::InvalidCloseTag(s, s2) => write!(f, "Invalid close tag: {} {}", s, s2),
            Error::UnclosedEntity(s) => write!(f, "Unclosed entity: {}", s),
            Error::InvalidEntity(s) => write!(f, "Invalid entity: {}", s),
            Error::UnknownPrefix(s) => write!(f, "Unknown prefix: {}", s),
            Error::DuplicateAttribute(s) => write!(f, "Duplicate attribute: {}", s),
            Error::UnsupportedVersion(s) => write!(f, "Unsupported version: {}", s),
            Error::UnsupportedEncoding(s) => write!(f, "Unsupported encoding: {}", s),
            Error::UnsupportedNotStandalone => write!(f, "Unsupported standalone"),
            Error::DtdUnsupported => write!(f, "DTD is not supported"),
            Error::IllegalAtTopLevel(_) => write!(f, "Illegal content under document node (attribute, namespace or document node"),
            Error::TextAtTopLevel(_) => write!(f, "Text node under document not. Not allowed in a well-formed document, but allowed in a fragment"),
            Error::NoElementAtTopLevel => write!(f, "No element under document root. Not allowed in a well-formed document, but allowed in a fragment"),
            Error::MultipleElementsAtTopLevel => write!(f, "Multiple elements under document root. Not allowed in a well-formed document, but allowed in a fragment"),
            Error::Parser(e) => write!(f, "Parser error: {}", e),
            Error::Io(s) => write!(f, "IO error: {}", s),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "Xot error"
    }
}
