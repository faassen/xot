use crate::{xotdata::Node, Span};

/// An error that occurred during parsing.
#[derive(Debug, Clone)]
pub enum ParseError {
    /// The XML is not well-formed - a tag is opened and never closed.
    UnclosedTag,
    /// The XML is not well-formed - a tag is closed that was never opened.
    InvalidCloseTag(String, String, Span),
    /// The XML is not well-formed - you use `&` to open an entity without
    /// closing it with `;`.
    UnclosedEntity(String),
    /// The entity is not known. Only the basic entities are supported
    /// right now, not any user defined ones.
    InvalidEntity(String),
    /// You used a namespace prefix that is not declared during parsing.
    UnknownPrefix(String, Span),
    /// You declared an attribute of the same name twice.
    DuplicateAttribute(String, Span),
    /// Unsupported XML version. Only 1.0 is supported.
    UnsupportedVersion(String, Span),
    /// Unsupported XML encoding. Only UTF-8 is supported.
    UnsupportedEncoding(String),
    /// Unsupported standalone declaration. Only `yes` is supported.
    UnsupportedNotStandalone(Span),
    /// XML DTD is not supported.
    DtdUnsupported(Span),

    /// xmlparser error
    XmlParser(xmlparser::Error, usize),
}

impl ParseError {
    /// Obtain the span for a ParseError.
    pub fn span(&self) -> Span {
        match self {
            ParseError::UnclosedTag => todo!(),
            ParseError::InvalidCloseTag(_, _, span) => *span,
            ParseError::UnclosedEntity(_) => todo!(),
            ParseError::InvalidEntity(_) => todo!(),
            ParseError::UnknownPrefix(_, span) => *span,
            ParseError::DuplicateAttribute(_, span) => *span,
            ParseError::UnsupportedVersion(_, span) => *span,
            ParseError::UnsupportedEncoding(_) => todo!(),
            ParseError::UnsupportedNotStandalone(span) => *span,
            ParseError::DtdUnsupported(span) => *span,
            ParseError::XmlParser(_, position) => Span::new(*position, *position),
        }
    }
}

/// Xot errors
#[derive(Debug, Clone)]
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

    /// An error during parsing
    Parse(ParseError),

    /// You used a namespace prefix that is not declared.
    ///
    /// Note that this error does not occur during parsing but during
    /// name creation.
    UnknownPrefix(String),

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

    /// IO error
    ///
    /// We take the string version of the IO error so as to keep errors comparable,
    /// which is more important than the exact error object in this case (serialization)
    Io(String),
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
        Error::Io(e.to_string())
    }
}

impl From<ParseError> for Error {
    #[inline]
    fn from(e: ParseError) -> Self {
        Error::Parse(e)
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
            Error::Parse(e) => write!(f, "Parse error: {:?}", e),
            Error::UnknownPrefix(s) => write!(f, "Unknown prefix: {}", s),
            Error::IllegalAtTopLevel(_) => write!(f, "Illegal content under document node (attribute, namespace or document node"),
            Error::TextAtTopLevel(_) => write!(f, "Text node under document not. Not allowed in a well-formed document, but allowed in a fragment"),
            Error::NoElementAtTopLevel => write!(f, "No element under document root. Not allowed in a well-formed document, but allowed in a fragment"),
            Error::MultipleElementsAtTopLevel => write!(f, "Multiple elements under document root. Not allowed in a well-formed document, but allowed in a fragment"),
            Error::Io(s) => write!(f, "IO error: {}", s),
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::UnclosedTag => write!(f, "Unclosed tag"),
            ParseError::InvalidCloseTag(s, s2, _) => write!(f, "Invalid close tag: {} {}", s, s2),
            ParseError::UnclosedEntity(s) => write!(f, "Unclosed entity: {}", s),
            ParseError::InvalidEntity(s) => write!(f, "Invalid entity: {}", s),
            ParseError::UnknownPrefix(s, _) => write!(f, "Unknown prefix: {}", s),
            ParseError::DuplicateAttribute(s, _) => write!(f, "Duplicate attribute: {}", s),
            ParseError::UnsupportedVersion(s, _) => write!(f, "Unsupported version: {}", s),
            ParseError::UnsupportedEncoding(s) => write!(f, "Unsupported encoding: {}", s),
            ParseError::UnsupportedNotStandalone(_) => write!(f, "Unsupported standalone"),
            ParseError::DtdUnsupported(_) => write!(f, "DTD is not supported"),
            ParseError::XmlParser(e, _position) => write!(f, "Parser error: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "Xot error"
    }
}
