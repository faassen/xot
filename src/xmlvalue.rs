use std::fmt::Debug;
use vector_map::VecMap;

use crate::error::Error;
use crate::name::NameId;
use crate::namespace::NamespaceId;
use crate::prefix::PrefixId;

/// The type of the XML node.
///
/// Access it using [`Value::value_type`] or
/// [`Xot::value_type`](crate::xotdata::Xot::value_type).
///
///    
/// The `ValueType` can be used if you are interested in
/// the type of the value without needing to match on it.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ValueType {
    /// Document root that holds everything.
    /// Note that this not the same as the document
    /// element.
    Root,
    /// Element; it has a name, attributes and namespace information.
    Element,
    /// Text. You can get and set the text value.
    Text,
    /// Processing instruction
    ProcessingInstruction,
    /// Comment.
    Comment,
}

/// An XML value.
///
/// Access it using [`Xot::value`](crate::xotdata::Xot::value) or
/// mutably using [`Xot::value_mut`](crate::xotdata::Xot::value_mut).
#[derive(Debug, Clone)]
pub enum Value {
    /// Document root that holds everything. Note that this not the same as the document
    /// element.
    Root,
    /// Element; it has a name, attributes and namespace information.
    Element(Element),
    /// Text. You can get and set the text value.
    Text(Text),
    /// Processing instruction.
    ProcessingInstruction(ProcessingInstruction),
    /// Comment.
    Comment(Comment),
}

impl Value {
    /// Returns the type of the XML value.
    pub fn value_type(&self) -> ValueType {
        match self {
            Value::Root => ValueType::Root,
            Value::Element(_) => ValueType::Element,
            Value::Text(_) => ValueType::Text,
            Value::Comment(_) => ValueType::Comment,
            Value::ProcessingInstruction(_) => ValueType::ProcessingInstruction,
        }
    }
}

/// A map of NameId to String for attributes
pub type Attributes = VecMap<NameId, String>;
/// A map of PrefixId to NamespaceId for namespace declarations.
pub type ToNamespace = VecMap<PrefixId, NamespaceId>;
pub(crate) type ToPrefix = VecMap<NamespaceId, PrefixId>;

#[derive(Debug, Clone)]
pub(crate) struct NamespaceInfo {
    pub(crate) to_namespace: ToNamespace,
    pub(crate) to_prefix: ToPrefix,
}

impl NamespaceInfo {
    pub(crate) fn new() -> Self {
        NamespaceInfo {
            to_namespace: VecMap::new(),
            to_prefix: VecMap::new(),
        }
    }

    pub(crate) fn add(&mut self, prefix_id: PrefixId, namespace_id: NamespaceId) {
        self.to_namespace.insert(prefix_id, namespace_id);
        self.to_prefix.insert(namespace_id, prefix_id);
    }

    pub(crate) fn remove_by_namespace_id(&mut self, namespace_id: NamespaceId) {
        if let Some(prefix_id) = self.to_prefix.remove(&namespace_id) {
            self.to_namespace.remove(&prefix_id);
        }
    }

    pub(crate) fn remove_by_prefix_id(&mut self, prefix_id: PrefixId) {
        if let Some(namespace_id) = self.to_namespace.remove(&prefix_id) {
            self.to_prefix.remove(&namespace_id);
        }
    }
}

/// XML element value.
///
/// Example: `<foo/>` or `<foo bar="baz"/>`.
#[derive(Debug, Clone)]
pub struct Element {
    pub(crate) name_id: NameId,
    pub(crate) attributes: Attributes,
    pub(crate) namespace_info: NamespaceInfo,
}

impl Element {
    pub(crate) fn new(name_id: NameId) -> Self {
        Element {
            name_id,
            attributes: Attributes::new(),
            namespace_info: NamespaceInfo::new(),
        }
    }

    /// The name of the element.
    pub fn name(&self) -> NameId {
        self.name_id
    }

    /// The attributes of the element.
    pub fn attributes(&self) -> &Attributes {
        &self.attributes
    }

    /// Get an attribute by name.
    pub fn get_attribute(&self, name_id: NameId) -> Option<&str> {
        self.attributes.get(&name_id).map(|s| s.as_str())
    }

    /// Set an attribute value.
    pub fn set_attribute<S: Into<String>>(&mut self, name_id: NameId, value: S) {
        self.attributes.insert(name_id, value.into());
    }

    /// Remove an attribute.
    pub fn remove_attribute(&mut self, name_id: NameId) {
        self.attributes.remove(&name_id);
    }

    /// Add a prefix to namespace mapping.
    pub fn set_prefix(&mut self, prefix_id: PrefixId, namespace_id: NamespaceId) {
        self.namespace_info.add(prefix_id, namespace_id);
    }

    /// Remove namespace prefix and associated namespace.
    ///
    /// This may result in documents with missing prefixes.
    pub fn remove_prefix(&mut self, prefix_id: PrefixId) {
        self.namespace_info.remove_by_prefix_id(prefix_id);
    }

    /// Get the prefix for a namespace, if defined on this element.
    ///
    /// This does not check for ancestor namespace definitions.
    pub fn get_prefix(&self, namespace_id: NamespaceId) -> Option<PrefixId> {
        self.namespace_info.to_prefix.get(&namespace_id).copied()
    }

    /// Get the namespace for a prefix, if defined on this element.
    ///
    /// This does not check for ancestor namespace definitions.
    pub fn get_namespace(&self, prefix_id: PrefixId) -> Option<NamespaceId> {
        self.namespace_info.to_namespace.get(&prefix_id).copied()
    }

    /// Get a map of prefixes to namespaces.
    ///
    /// It only returns those prefixes that are defined
    /// on this element.
    pub fn prefixes(&self) -> &ToNamespace {
        &self.namespace_info.to_namespace
    }
}

/// XML text value.
///
/// Example: `Bar` in `<foo>Bar</foo>`, or `hello` and `world` in `<greeting>hello<sep/>world</greeting>`.
#[derive(Debug, Clone)]
pub struct Text {
    pub(crate) text: String,
}

impl Text {
    pub(crate) fn new(text: String) -> Self {
        Text { text }
    }

    /// Get the text value.
    pub fn get(&self) -> &str {
        &self.text
    }

    /// Set the text value.
    pub fn set<S: Into<String>>(&mut self, text: S) {
        self.text = text.into();
    }
}

/// XML comment.
///
/// Example: `<!-- foo -->`.
#[derive(Debug, Clone)]
pub struct Comment {
    pub(crate) text: String,
}

impl Comment {
    pub(crate) fn new(text: String) -> Self {
        Comment { text }
    }

    /// Get the comment text.
    pub fn get(&self) -> &str {
        &self.text
    }

    /// Set the comment text. Rejects
    /// comments that contain `--` as illegal.
    pub fn set<S: Into<String>>(&mut self, text: S) -> Result<(), Error> {
        let text = text.into();
        if text.contains("--") {
            return Err(Error::InvalidComment(text));
        }
        self.text = text;
        Ok(())
    }
}

/// XML processing instruction value.
///
/// Example: `<?foo?>` or `<?foo bar?>`.
#[derive(Debug, Clone)]
pub struct ProcessingInstruction {
    pub(crate) target: String,
    pub(crate) data: Option<String>,
}

impl ProcessingInstruction {
    pub(crate) fn new(target: String, data: Option<String>) -> Self {
        ProcessingInstruction { target, data }
    }

    /// Get processing instruction target.
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Get processing instruction data.
    pub fn data(&self) -> Option<&str> {
        self.data.as_deref()
    }

    /// Set target. Rejects any target that is
    /// the string `"xml"` (or case variations) as it's reserved for XML.
    pub fn set_target<S: Into<String>>(&mut self, target: S) -> Result<(), Error> {
        let target = target.into();
        if target.to_lowercase() == "xml" {
            return Err(Error::InvalidTarget(target));
        }
        // XXX Ideally check that name follows XML spec
        self.target = target;
        Ok(())
    }

    /// Set data.
    pub fn set_data<S: Into<String>>(&mut self, data: Option<S>) {
        // XXX Ideally check that data follows XML spec, i.e. not contain
        // "?>".
        self.data = data.map(|s| s.into());
    }
}
