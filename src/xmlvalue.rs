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
pub type Prefixes = VecMap<PrefixId, NamespaceId>;

/// XML element value.
///
/// Example: `<foo/>` or `<foo bar="baz"/>`.
#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub(crate) name_id: NameId,
    pub(crate) prefixes: Prefixes,
    pub(crate) attributes: Attributes,
}

impl Element {
    pub(crate) fn new(name_id: NameId) -> Self {
        Element {
            name_id,
            prefixes: Prefixes::new(),
            attributes: Attributes::new(),
        }
    }

    /// The name of the element.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let name_doc = xot.add_name("doc");
    ///
    /// let root = xot.parse("<doc/>")?;
    /// let doc_el = xot.document_element(root).unwrap();
    /// let element = xot.element(doc_el).unwrap();
    /// assert_eq!(element.name(), name_doc);
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn name(&self) -> NameId {
        self.name_id
    }

    /// The attributes of the element.
    ///
    /// ```rust
    /// use xot::{Xot, Attributes};
    ///
    /// let mut xot = Xot::new();
    /// let name_a = xot.add_name("a");
    /// let name_b = xot.add_name("b");
    ///
    /// let root = xot.parse(r#"<doc a="A" b="B" />"#)?;
    /// let doc_el = xot.document_element(root).unwrap();
    /// let element = xot.element(doc_el).unwrap();
    ///
    /// let mut expected = Attributes::new();
    /// expected.insert(name_a, "A".to_string());
    /// expected.insert(name_b, "B".to_string());
    ///
    /// assert_eq!(element.attributes(), &expected);
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn attributes(&self) -> &Attributes {
        &self.attributes
    }

    /// Get an attribute by name.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let name_a = xot.add_name("a");
    ///
    /// let root = xot.parse(r#"<doc a="A" />"#)?;
    /// let doc_el = xot.document_element(root).unwrap();
    /// let element = xot.element(doc_el).unwrap();
    ///
    /// assert_eq!(element.get_attribute(name_a), Some("A"));
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn get_attribute(&self, name_id: NameId) -> Option<&str> {
        self.attributes.get(&name_id).map(|s| s.as_str())
    }

    /// Set an attribute value.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let name_a = xot.add_name("a");
    ///
    /// let root = xot.parse(r#"<doc/>"#)?;
    /// let doc_el = xot.document_element(root).unwrap();
    /// let element = xot.element_mut(doc_el).unwrap();
    ///
    /// element.set_attribute(name_a, "A");
    ///
    /// assert_eq!(element.get_attribute(name_a), Some("A"));
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn set_attribute<S: Into<String>>(&mut self, name_id: NameId, value: S) {
        self.attributes.insert(name_id, value.into());
    }

    /// Remove an attribute.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let name_a = xot.add_name("a");
    ///
    /// let root = xot.parse(r#"<doc a="A" />"#)?;
    /// let doc_el = xot.document_element(root).unwrap();
    /// let element = xot.element_mut(doc_el).unwrap();
    ///
    /// element.remove_attribute(name_a);
    ///
    /// assert_eq!(element.get_attribute(name_a), None);
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn remove_attribute(&mut self, name_id: NameId) {
        self.attributes.remove(&name_id);
    }

    /// Add a prefix to namespace mapping.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let prefix_x = xot.add_prefix("x");
    /// let namespace_x = xot.add_namespace("http://example.com/x");
    ///
    /// let root = xot.parse(r#"<doc/>"#)?;
    /// let doc_el = xot.document_element(root).unwrap();
    /// let element = xot.element_mut(doc_el).unwrap();
    ///
    /// element.set_prefix(prefix_x, namespace_x);
    ///
    /// assert_eq!(element.prefixes().iter().collect::<Vec<_>>(), [(&prefix_x, &namespace_x)]);
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn set_prefix(&mut self, prefix_id: PrefixId, namespace_id: NamespaceId) {
        self.prefixes.insert(prefix_id, namespace_id);
    }

    /// Remove namespace prefix and associated namespace.
    ///
    /// This may result in documents with missing prefixes. This can be safely
    /// serialized if you call [`Xot::create_missing_prefixes`] before serialization.
    pub fn remove_prefix(&mut self, prefix_id: PrefixId) {
        self.prefixes.remove(&prefix_id);
    }

    /// Remove prefixs by namespace.
    ///
    /// This may result in documents with missing prefixes. This can be safely
    /// serialized if you call [`Xot::create_missing_prefixes`] before
    /// serialization.
    pub fn remove_namespace(&mut self, namespace_id: NamespaceId) {
        self.prefixes.retain(|_, v| *v != namespace_id);
    }

    /// Get the namespace for a prefix, if defined on this element.
    ///
    /// This does not check for ancestor namespace definitions.
    pub fn get_namespace(&self, prefix_id: PrefixId) -> Option<NamespaceId> {
        self.prefixes.get(&prefix_id).copied()
    }

    /// Get a map of prefixes to namespaces.
    ///
    /// It only returns those prefixes that are defined
    /// on this element.
    pub fn prefixes(&self) -> &Prefixes {
        &self.prefixes
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
    ///
    /// See [`Xot::text_str`](`crate::Xot::text_str`) and [`Xot::text_content_str`](`crate::Xot::text_content_str`) for
    /// more convenient ways to get text values.
    pub fn get(&self) -> &str {
        &self.text
    }

    /// Set the text value.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse(r#"<doc>Example</doc>"#)?;
    /// let doc_el = xot.document_element(root).unwrap();
    /// let text_node = xot.first_child(doc_el).unwrap();
    ///
    /// let text = xot.text_mut(text_node).unwrap();
    /// text.set("New text");
    ///
    /// assert_eq!(xot.to_string(root).unwrap(), r#"<doc>New text</doc>"#);
    /// # Ok::<(), xot::Error>(())
    /// ```
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

    /// Set the comment text.
    ///
    /// Rejects comments that contain `--` as illegal.
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

    /// Set target.
    ///
    /// Rejects any target that is the string `"xml"` (or case variations) as
    /// it's reserved for XML.
    pub fn set_target<S: Into<String>>(&mut self, target: S) -> Result<(), Error> {
        let target = target.into();
        if target.to_lowercase() == "xml" {
            return Err(Error::InvalidTarget(target));
        }
        if target.is_empty() {
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
        if let Some(data) = data {
            let data = data.into();
            if !data.is_empty() {
                self.data = Some(data);
                return;
            }
        }
        self.data = None;
    }
}
