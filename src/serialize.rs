use std::io::Write;

use crate::serializer::{Serializer, XmlSerializerWriter};

use crate::error::Error;
use crate::xmlvalue::{ToNamespace, ValueType};
use crate::xotdata::{Node, Xot};

/// ## Serialization
impl<'a> Xot<'a> {
    /// Serialize document to a writer.
    ///
    /// This only works with a root node.
    ///
    /// ```rust
    ///
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse("<p>Example</p>")?;
    ///
    /// let mut buf = Vec::new();
    /// xot.serialize(root, &mut buf);
    ///
    /// assert_eq!(buf, b"<p>Example</p>");
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn serialize(&mut self, node: Node, w: &mut impl Write) {
        let root_element = self.document_element(node).unwrap();
        self.create_missing_prefixes(root_element).unwrap();
        self.serialize_or_missing_prefix(node, w).unwrap();
    }

    /// Serialize a node to a writer.
    ///
    /// This works with any node and produces an XML fragment for this node. If
    /// the node is an element, any prefixes needed for the fragment are added
    /// to this element.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse("<doc><p>Example</p></doc>")?;
    /// let doc_el = xot.document_element(root).unwrap();
    /// let p = xot.first_child(doc_el).unwrap();

    /// let mut buf = Vec::new();
    /// xot.serialize_node(p, &mut buf);
    /// assert_eq!(buf, b"<p>Example</p>");
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    ///
    /// Prefixes defined higher up are automatically serialized:
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse("<doc xmlns:foo='http://example.com'><p>Example</p></doc>")?;
    /// let doc_el = xot.document_element(root).unwrap();
    /// let p = xot.first_child(doc_el).unwrap();
    ///
    /// let mut buf = Vec::new();
    /// xot.serialize_node(p, &mut buf);
    /// assert_eq!(buf, b"<p xmlns:foo=\"http://example.com\">Example</p>");
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn serialize_node(&mut self, node: Node, w: &mut impl Write) {
        let root_element = self.top_element(node);
        self.create_missing_prefixes(root_element).unwrap();
        // collect namespace prefixes for all ancestors of the fragment
        let extra_prefixes = if let Some(parent) = self.parent(node) {
            if self.value_type(parent) != ValueType::Root {
                self.to_namespace_in_scope(parent)
            } else {
                ToNamespace::new()
            }
        } else {
            ToNamespace::new()
        };
        // now serialize with those additional prefixes
        self.serialize_node_helper(node, w, &extra_prefixes)
            .unwrap();
    }

    /// Serialize document and fail if namespaces encountered without prefix defined.
    ///
    /// This fails if there is a namespace without a prefix. Use
    /// [`Xot::serialize`] if you want it to generate synthetic prefixes
    /// instead.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let ns = xot.add_namespace("http://example.com");
    /// let doc_name = xot.add_name_ns("doc", ns);
    /// let doc_el = xot.new_element(doc_name);
    /// let root = xot.new_root(doc_el)?;
    ///
    /// // we never define a prefix
    ///
    /// let mut buf = Vec::new();
    /// assert!(xot.serialize_or_missing_prefix(root, &mut buf).is_err());
    ///
    /// // if we define the prefix, it's fine
    /// let prefix = xot.add_prefix("foo");
    /// let doc_value = xot.element_mut(doc_el).unwrap();
    /// doc_value.set_prefix(prefix, ns);
    ///
    /// let mut buf = Vec::new();
    /// assert!(xot.serialize_or_missing_prefix(root, &mut buf).is_ok());
    /// assert_eq!(buf, b"<foo:doc xmlns:foo=\"http://example.com\"/>");
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn serialize_or_missing_prefix(&self, node: Node, w: &mut impl Write) -> Result<(), Error> {
        if self.value_type(node) != ValueType::Root {
            panic!("Can only serialize root nodes");
        }
        let extra_prefixes = ToNamespace::new();
        self.serialize_node_helper(node, w, &extra_prefixes)
    }

    /// Serialize document to a string.
    ///
    /// Like [`Xot::serialize_or_missing_prefix`], but returns a string instead of writing to a writer.
    pub fn serialize_or_missing_prefix_to_string(&self, node: Node) -> Result<String, Error> {
        let mut buf = Vec::new();
        self.serialize_or_missing_prefix(node, &mut buf)?;
        Ok(String::from_utf8(buf).unwrap())
    }

    /// Serialize document to a string.
    ///
    /// This only works with a root node.
    pub fn serialize_to_string(&mut self, node: Node) -> String {
        let mut buf = Vec::new();
        self.serialize(node, &mut buf);
        String::from_utf8(buf).unwrap()
    }

    /// Serialize a node to a string.
    ///
    /// This works with any node and produces an XML fragment.
    pub fn serialize_node_to_string(&mut self, node: Node) -> String {
        let mut buf = Vec::new();
        self.serialize_node(node, &mut buf);
        String::from_utf8(buf).unwrap()
    }

    fn serialize_node_helper(
        &self,
        node: Node,
        w: &mut impl Write,
        extra_prefixes: &ToNamespace,
    ) -> Result<(), Error> {
        let mut xml_writer = XmlSerializerWriter::new(self, w, extra_prefixes);
        let mut serializer = Serializer::new(&mut xml_writer, extra_prefixes.clone());
        serializer.serialize_node(self, node)
    }
}
