use std::fmt::Debug;

use ahash::AHashMap;

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
    /// Attribute
    Attribute,
    /// Namespace
    Namespace,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) enum ValueCategory {
    Normal,
    Attribute,
    Namespace,
}

/// An XML value.
///
/// Access it using [`Xot::value`](crate::xotdata::Xot::value) or
/// mutably using [`Xot::value_mut`](crate::xotdata::Xot::value_mut).
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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
    /// Attribute
    Attribute(Attribute),
    /// Namespace
    Namespace(Namespace),
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
            Value::Attribute(_) => ValueType::Attribute,
            Value::Namespace(_) => ValueType::Namespace,
        }
    }

    pub(crate) fn value_category(&self) -> ValueCategory {
        match self {
            Value::Root
            | Value::Element(_)
            | Value::Text(_)
            | Value::ProcessingInstruction(_)
            | Value::Comment(_) => ValueCategory::Normal,
            Value::Attribute(_) => ValueCategory::Attribute,
            Value::Namespace(_) => ValueCategory::Namespace,
        }
    }

    pub(crate) fn is_normal(&self) -> bool {
        matches!(
            self,
            Value::Root
                | Value::Element(_)
                | Value::Text(_)
                | Value::ProcessingInstruction(_)
                | Value::Comment(_)
        )
    }
}

/// A map of PrefixId to NamespaceId for namespace tracking.
pub type Prefixes = AHashMap<PrefixId, NamespaceId>;

/// XML element name.
///
/// Does not include namespace or attribute information;
/// this is kept in the tree.
///
/// Example: `<foo/>`.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Element {
    pub(crate) name_id: NameId,
}

impl Element {
    pub(crate) fn new(name_id: NameId) -> Self {
        Self { name_id }
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

    // /// Compare with other element for semantic equality.
    // ///
    // /// This ignores element prefixes.
    // pub fn compare(&self, other: &Element) -> bool {
    //     self.advanced_compare(other, |a, b| a == b)
    // }

    // /// Compare with other element for semantic equality.
    // ///
    // /// You configure this with a function that compares attribute text.
    // ///
    // /// This ignores element prefixes.
    // pub fn advanced_compare<C>(&self, other: &Element, text_compare: C) -> bool
    // where
    //     C: Fn(&str, &str) -> bool,
    // {
    //     if self.name() != other.name() {
    //         return false;
    //     }
    //     let self_attributes = self.attributes();
    //     let other_attributes = other.attributes();
    //     if self_attributes.len() != other_attributes.len() {
    //         return false;
    //     }
    //     // if we can't find a value for a key in a in b, then we
    //     // know they aren't the same, given we already compared the length
    //     for (key, value_a) in self_attributes {
    //         let value_b = other_attributes.get(key);
    //         if let Some(value_b) = value_b {
    //             if !text_compare(value_a, value_b) {
    //                 return false;
    //             }
    //         } else {
    //             return false;
    //         }
    //     }
    //     true
    // }

    // /// Compare with other element for semantic equality, ignoring particular
    // /// attributes in the comparison.
    // ///
    // /// This ignores element prefixes.
    // pub fn compare_ignore_attributes(&self, other: &Element, ignore_attributes: &[NameId]) -> bool {
    //     if self.name() != other.name() {
    //         return false;
    //     }
    //     // count the amount of attributes we compare
    //     let mut compare_attributes_count = 0;

    //     let self_attributes = self.attributes();
    //     let other_attributes = other.attributes();

    //     for (key, value_a) in self_attributes {
    //         if ignore_attributes.contains(key) {
    //             continue;
    //         }
    //         let value_b = other_attributes.get(key);
    //         if Some(value_a) != value_b {
    //             return false;
    //         }
    //         compare_attributes_count += 1;
    //     }

    //     let mut other_ignore_attributes = 0;
    //     for ignore_attribute in ignore_attributes {
    //         if other_attributes.get(ignore_attribute).is_some() {
    //             other_ignore_attributes += 1;
    //         }
    //     }
    //     // we expect the amount of non-ignored attributes in self to
    //     // be the same as the amount of non-ignored attributes in other
    //     compare_attributes_count == other_attributes.len() - other_ignore_attributes
    // }
}

/// XML text value.
///
/// Example: `Bar` in `<foo>Bar</foo>`, or `hello` and `world` in `<greeting>hello<sep/>world</greeting>`.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Attribute {
    pub(crate) name_id: NameId,
    pub(crate) value: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Namespace {
    pub(crate) prefix_id: PrefixId,
    pub(crate) namespace_id: NamespaceId,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xotdata::Xot;

    #[test]
    fn test_element_hashable_name() {
        let mut xot = Xot::new();
        let a = xot.add_name("a");
        let b = xot.add_name("b");

        let alpha = Element { name_id: a };
        let beta = Element { name_id: a };
        let gamma = Element { name_id: b };

        let hash_builder = ahash::RandomState::with_seed(42);
        let alpha_hash = hash_builder.hash_one(alpha);
        let beta_hash = hash_builder.hash_one(beta);
        let gamma_hash = hash_builder.hash_one(gamma);
        assert_eq!(alpha_hash, beta_hash);
        assert_ne!(alpha_hash, gamma_hash);
    }

    // #[test]
    // fn test_element_compare_same() {
    //     let mut xot = Xot::new();
    //     let a = xot.add_name("a");

    //     let mut alpha = Element {
    //         name_id: a,
    //         prefixes: Prefixes::new(),
    //         attributes: Attributes::new(),
    //     };
    //     let mut beta = Element {
    //         name_id: a,
    //         prefixes: Prefixes::new(),
    //         attributes: Attributes::new(),
    //     };
    //     alpha.set_attribute(a, "foo");
    //     beta.set_attribute(a, "foo");

    //     assert!(alpha.compare(&beta));
    // }

    // #[test]
    // fn test_element_compare_different_value() {
    //     let mut xot = Xot::new();
    //     let a = xot.add_name("a");

    //     let mut alpha = Element {
    //         name_id: a,
    //         prefixes: Prefixes::new(),
    //         attributes: Attributes::new(),
    //     };
    //     let mut beta = Element {
    //         name_id: a,
    //         prefixes: Prefixes::new(),
    //         attributes: Attributes::new(),
    //     };
    //     alpha.set_attribute(a, "foo");
    //     beta.set_attribute(a, "bar");

    //     assert!(!alpha.compare(&beta));
    // }

    // #[test]
    // fn test_element_compare_overlap() {
    //     let mut xot = Xot::new();
    //     let a = xot.add_name("a");
    //     let b = xot.add_name("b");

    //     let mut alpha = Element {
    //         name_id: a,
    //         prefixes: Prefixes::new(),
    //         attributes: Attributes::new(),
    //     };
    //     let mut beta = Element {
    //         name_id: a,
    //         prefixes: Prefixes::new(),
    //         attributes: Attributes::new(),
    //     };
    //     alpha.set_attribute(a, "foo");
    //     beta.set_attribute(a, "foo");
    //     beta.set_attribute(b, "bar");

    //     assert!(!alpha.compare(&beta));
    // }

    // #[test]
    // fn test_element_compare_ignore_attributes_same_ignorable_in_self() {
    //     let mut xot = Xot::new();
    //     let a = xot.add_name("a");
    //     let b = xot.add_name("b");

    //     let mut alpha = Element {
    //         name_id: a,
    //         prefixes: Prefixes::new(),
    //         attributes: Attributes::new(),
    //     };
    //     let mut beta = Element {
    //         name_id: a,
    //         prefixes: Prefixes::new(),
    //         attributes: Attributes::new(),
    //     };
    //     alpha.set_attribute(a, "foo");
    //     beta.set_attribute(a, "foo");
    //     alpha.set_attribute(b, "bar");
    //     assert!(alpha.compare_ignore_attributes(&beta, &[b]));
    //     assert!(!alpha.compare_ignore_attributes(&beta, &[]));
    // }

    // #[test]
    // fn test_element_compare_ignore_attributes_same_ignorable_in_other() {
    //     let mut xot = Xot::new();
    //     let a = xot.add_name("a");
    //     let b = xot.add_name("b");

    //     let mut alpha = Element {
    //         name_id: a,
    //         prefixes: Prefixes::new(),
    //         attributes: Attributes::new(),
    //     };
    //     let mut beta = Element {
    //         name_id: a,
    //         prefixes: Prefixes::new(),
    //         attributes: Attributes::new(),
    //     };
    //     alpha.set_attribute(a, "foo");
    //     beta.set_attribute(a, "foo");
    //     beta.set_attribute(b, "bar");
    //     assert!(alpha.compare_ignore_attributes(&beta, &[b]));
    //     assert!(!alpha.compare_ignore_attributes(&beta, &[]));
    // }

    // #[test]
    // fn test_element_compare_ignore_attributes_different_value() {
    //     let mut xot = Xot::new();
    //     let a = xot.add_name("a");
    //     let b = xot.add_name("b");

    //     let mut alpha = Element {
    //         name_id: a,
    //         prefixes: Prefixes::new(),
    //         attributes: Attributes::new(),
    //     };
    //     let mut beta = Element {
    //         name_id: a,
    //         prefixes: Prefixes::new(),
    //         attributes: Attributes::new(),
    //     };
    //     alpha.set_attribute(a, "foo");
    //     beta.set_attribute(a, "qux");
    //     beta.set_attribute(b, "bar");
    //     assert!(!alpha.compare_ignore_attributes(&beta, &[b]));
    //     assert!(!alpha.compare_ignore_attributes(&beta, &[]));
    // }

    // #[test]
    // fn test_element_compare_ignore_attributes_ignorable_in_both() {
    //     let mut xot = Xot::new();
    //     let a = xot.add_name("a");
    //     let b = xot.add_name("b");

    //     let mut alpha = Element {
    //         name_id: a,
    //         prefixes: Prefixes::new(),
    //         attributes: Attributes::new(),
    //     };
    //     let mut beta = Element {
    //         name_id: a,
    //         prefixes: Prefixes::new(),
    //         attributes: Attributes::new(),
    //     };
    //     alpha.set_attribute(a, "foo");
    //     alpha.set_attribute(b, "qux");
    //     beta.set_attribute(a, "foo");
    //     beta.set_attribute(b, "bar");
    //     assert!(alpha.compare_ignore_attributes(&beta, &[b]));
    //     assert!(!alpha.compare_ignore_attributes(&beta, &[]));
    // }
}
