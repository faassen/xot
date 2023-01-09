use std::io::Write;

use crate::serializer::{Serializer, SerializerWriter, XmlSerializerWriter};

use crate::error::Error;
use crate::xotdata::{Node, Xot};

/// ## Serialization
impl<'a> Xot<'a> {
    /// Serialize document to a Write.
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
    /// xot.serialize(root, &mut buf).unwrap();
    ///
    /// assert_eq!(buf, b"<p>Example</p>");
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn serialize(&mut self, node: Node, w: &mut impl Write) -> Result<(), Error> {
        let root_element = self.document_element(node).unwrap();
        self.create_missing_prefixes(root_element).unwrap();
        let mut serializer_writer = XmlSerializerWriter::new(self, w);
        let mut serializer = Serializer::new(self, &mut serializer_writer);
        serializer.serialize_node(node)
    }

    /// Serialize a node to a Write.
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
    pub fn serialize_node(&mut self, node: Node, w: &mut impl Write) -> Result<(), Error> {
        let root_element = self.top_element(node);
        self.create_missing_prefixes(root_element).unwrap();
        let mut serializer_writer = XmlSerializerWriter::new(self, w);
        let mut serializer = Serializer::new(self, &mut serializer_writer);
        serializer.serialize_node(node)
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
        let mut serializer_writer = XmlSerializerWriter::new(self, w);
        let mut serializer = Serializer::new(self, &mut serializer_writer);
        serializer.serialize_node(node)
    }

    /// Serialize document to a string.
    ///
    /// This only works with a root node.
    pub fn serialize_to_string(&mut self, node: Node) -> String {
        let mut buf = Vec::new();
        self.serialize(node, &mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    /// Serialize document to a string.
    ///
    /// Like [`Xot::serialize_or_missing_prefix`], but returns a string instead of writing to a writer.
    pub fn serialize_or_missing_prefix_to_string(&self, node: Node) -> Result<String, Error> {
        let mut buf = Vec::new();
        self.serialize_or_missing_prefix(node, &mut buf)?;
        Ok(String::from_utf8(buf).unwrap())
    }

    /// Serialize a node to a string.
    ///
    /// This works with any node and produces an XML fragment.
    pub fn serialize_node_to_string(&mut self, node: Node) -> String {
        let mut buf = Vec::new();
        self.serialize_node(node, &mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    /// Serialize node with a custom serializer writer.
    ///
    /// This is an advanced method that allows customisation of the XML writing.
    ///
    /// Note that this does not create missing prefixes; you need to call
    /// [`Xot::create_missing_prefixes`] yourself if you want it to create them.
    pub fn serialize_with_writer(
        &self,
        node: Node,
        serializer_writer: &mut impl SerializerWriter,
    ) -> Result<(), Error> {
        let mut serializer = Serializer::new(self, serializer_writer);
        serializer.serialize_node(node)
    }
}
