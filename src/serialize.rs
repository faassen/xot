use std::io::Write;

use crate::access::NodeEdge;
use crate::entity::{serialize_attribute, serialize_text};
use crate::error::Error;
use crate::fullname::FullnameSerializer;
use crate::namespace::NamespaceId;
use crate::prefix::PrefixId;
use crate::xmlvalue::{ToNamespace, Value, ValueType};
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
        let to_namespace = if let Some(parent) = self.parent(node) {
            if self.value_type(parent) != ValueType::Root {
                self.to_namespace_in_scope(parent)
            } else {
                ToNamespace::new()
            }
        } else {
            ToNamespace::new()
        };
        // now serialize with those additional prefixes
        self.serialize_node_helper(node, w, to_namespace).unwrap();
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
        self.serialize_node_helper(node, w, ToNamespace::new())
    }

    fn serialize_node_helper(
        &self,
        node: Node,
        w: &mut impl Write,
        to_namespace: ToNamespace,
    ) -> Result<(), Error> {
        let mut fullname_serializer = FullnameSerializer::with_to_namespace(to_namespace, self);
        for edge in self.traverse(node) {
            match edge {
                NodeEdge::Start(current_node) => {
                    self.handle_edge_start(node, current_node, w, &mut fullname_serializer)?;
                }
                NodeEdge::End(current_node) => {
                    self.handle_edge_end(current_node, w, &mut fullname_serializer)?;
                }
            }
        }
        Ok(())
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

    fn handle_edge_start(
        &self,
        top_node: Node,
        node: Node,
        w: &mut impl Write,
        fullname_serializer: &mut FullnameSerializer,
    ) -> Result<(), Error> {
        let value = self.value(node);
        match value {
            Value::Root => {}
            Value::Element(element) => {
                fullname_serializer.push(&element.namespace_info.to_namespace);

                let fullname = fullname_serializer.fullname_or_err(element.name_id)?;

                write!(w, "<{}", fullname)?;
                // serialize any extra prefixes if this is the top element of
                // a fragment and they aren't declared already
                if node == top_node {
                    for (prefix_id, namespace_id) in fullname_serializer.top_to_namespace().iter() {
                        if !element.namespace_info.to_namespace.contains_key(prefix_id) {
                            self.write_namespace_declaration(*prefix_id, *namespace_id, w)?;
                        }
                    }
                }
                for (prefix_id, namespace_id) in element.namespace_info.to_namespace.iter() {
                    self.write_namespace_declaration(*prefix_id, *namespace_id, w)?;
                }
                for (name_id, value) in element.attributes.iter() {
                    let fullname = fullname_serializer.fullname_attr_or_err(*name_id)?;
                    write!(w, " {}=\"{}\"", fullname, serialize_attribute(value.into()))?;
                }

                if self.first_child(node).is_none() {
                    write!(w, "/>")?;
                } else {
                    write!(w, ">")?;
                }
            }
            Value::Text(text) => {
                write!(w, "{}", serialize_text(text.get().into()))?;
            }
            Value::Comment(comment) => {
                write!(w, "<!--{}-->", comment.get())?;
            }
            Value::ProcessingInstruction(pi) => {
                if let Some(data) = pi.data() {
                    if !data.is_empty() {
                        write!(w, "<?{} {}?>", pi.target(), data)?;
                    } else {
                        write!(w, "<?{}?>", pi.target())?;
                    }
                } else {
                    write!(w, "<?{}?>", pi.target())?;
                }
            }
        }
        Ok(())
    }

    fn handle_edge_end(
        &self,
        node: Node,
        w: &mut impl Write,
        fullname_serializer: &mut FullnameSerializer,
    ) -> Result<(), Error> {
        let value = self.value(node);
        if let Value::Element(element) = value {
            if self.first_child(node).is_some() {
                let fullname = fullname_serializer.fullname_or_err(element.name_id)?;
                write!(w, "</{}>", fullname)?;
            }
            fullname_serializer.pop(&element.namespace_info.to_namespace);
        }
        Ok(())
    }

    fn write_namespace_declaration(
        &self,
        prefix_id: PrefixId,
        namespace_id: NamespaceId,
        w: &mut impl Write,
    ) -> Result<(), Error> {
        let namespace = self.namespace_lookup.get_value(namespace_id);
        if prefix_id == self.empty_prefix_id {
            write!(w, " xmlns=\"{}\"", namespace)?;
        } else {
            write!(
                w,
                " xmlns:{}=\"{}\"",
                self.prefix_lookup.get_value(prefix_id),
                namespace
            )?;
        }
        Ok(())
    }
}
