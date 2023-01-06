use std::io::Write;

use crate::access::NodeEdge;
use crate::entity::{serialize_attribute, serialize_text};
use crate::error::Error;
use crate::fullname::FullnameSerializer;
use crate::namespace::NamespaceId;
use crate::prefix::PrefixId;
use crate::xmlvalue::{Element, ToNamespace, Value, ValueType};
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
        to_namespace: ToNamespace,
    ) -> Result<(), Error> {
        let mut fullname_serializer = FullnameSerializer::with_to_namespace(to_namespace, self);
        let mut xml_writer = XmlSerializerWriter::new(self, &mut fullname_serializer, w);

        for edge in self.traverse(node) {
            match edge {
                NodeEdge::Start(current_node) => {
                    self.handle_edge_start(node, current_node, &mut xml_writer)?;
                }
                NodeEdge::End(current_node) => {
                    self.handle_edge_end(current_node, &mut xml_writer)?;
                }
            }
        }
        Ok(())
    }

    fn handle_edge_start(
        &self,
        top_node: Node,
        node: Node,
        xml_writer: &mut XmlSerializerWriter<'a, impl Write>,
    ) -> Result<(), Error> {
        let value = self.value(node);
        match value {
            Value::Root => {}
            Value::Element(element) => {
                xml_writer.write_start_tag_open(node, element)?;

                // serialize any extra prefixes if this is the top element of
                // a fragment and they aren't declared already
                if node == top_node {
                    let extra_prefixes = xml_writer
                        .extra_prefixes()
                        .iter()
                        .map(|(prefix_id, namespace_id)| (*prefix_id, *namespace_id))
                        .collect::<Vec<_>>();
                    for (prefix_id, namespace_id) in extra_prefixes {
                        if !element.namespace_info.to_namespace.contains_key(&prefix_id) {
                            xml_writer.write_space()?;
                            xml_writer.write_namespace_declaration(
                                node,
                                element,
                                prefix_id,
                                namespace_id,
                            )?;
                        }
                    }
                }

                for (prefix_id, namespace_id) in element.prefixes() {
                    xml_writer.write_space()?;
                    xml_writer.write_namespace_declaration(
                        node,
                        element,
                        *prefix_id,
                        *namespace_id,
                    )?;
                }

                for (name_id, value) in element.attributes() {
                    xml_writer.write_space()?;
                    xml_writer.write_attribute(node, element, *name_id, value)?;
                }

                xml_writer.write_start_tag_close(node, element)?;
            }
            Value::Text(text) => {
                xml_writer.write_text(node, text)?;
            }
            Value::Comment(comment) => {
                xml_writer.write_comment(node, comment)?;
            }
            Value::ProcessingInstruction(pi) => {
                xml_writer.write_processing_instruction(node, pi)?;
            }
        }
        Ok(())
    }

    fn handle_edge_end(
        &self,
        node: Node,
        xml_writer: &mut XmlSerializerWriter<'a, impl Write>,
    ) -> Result<(), Error> {
        let value = self.value(node);
        if let Value::Element(element) = value {
            xml_writer.write_end_tag(node, element)?;
        }
        Ok(())
    }
}

use crate::name::NameId;
use crate::xmlvalue::{Comment, ProcessingInstruction, Text};

pub trait SerializerWriter {
    fn extra_prefixes(&self) -> &ToNamespace;
    fn write_start_tag_open(&mut self, node: Node, element: &Element) -> Result<(), Error>;
    fn write_start_tag_close(&mut self, node: Node, element: &Element) -> Result<(), Error>;
    fn write_end_tag(&mut self, node: Node, element: &Element) -> Result<(), Error>;
    fn write_namespace_declaration(
        &mut self,
        node: Node,
        element: &Element,
        prefix_id: PrefixId,
        namespace_id: NamespaceId,
    ) -> Result<(), Error>;
    fn write_attribute(
        &mut self,
        node: Node,
        element: &Element,
        name_id: NameId,
        value: &str,
    ) -> Result<(), Error>;
    fn write_text(&mut self, node: Node, text: &Text) -> Result<(), Error>;
    fn write_comment(&mut self, node: Node, comment: &Comment) -> Result<(), Error>;
    fn write_processing_instruction(
        &mut self,
        node: Node,
        pi: &ProcessingInstruction,
    ) -> Result<(), Error>;
    fn write_space(&mut self) -> Result<(), Error>;
}

struct XmlSerializerWriter<'a, W: Write> {
    xot: &'a Xot<'a>,
    fullname_serializer: &'a mut FullnameSerializer<'a>,
    w: &'a mut W,
}

impl<'a, W: Write> XmlSerializerWriter<'a, W> {
    pub(crate) fn new(
        xot: &'a Xot<'a>,
        fullname_serializer: &'a mut FullnameSerializer<'a>,
        w: &'a mut W,
    ) -> XmlSerializerWriter<'a, W> {
        XmlSerializerWriter {
            xot,
            fullname_serializer,
            w,
        }
    }
}

impl<'a, W: Write> SerializerWriter for XmlSerializerWriter<'a, W> {
    fn extra_prefixes(&self) -> &ToNamespace {
        self.fullname_serializer.top_to_namespace()
    }

    fn write_start_tag_open(&mut self, node: Node, element: &Element) -> Result<(), Error> {
        self.fullname_serializer
            .push(&element.namespace_info.to_namespace);
        write!(
            self.w,
            "<{}",
            self.fullname_serializer.fullname_or_err(element.name())?
        )?;
        Ok(())
    }

    fn write_start_tag_close(&mut self, node: Node, element: &Element) -> Result<(), Error> {
        if self.xot.first_child(node).is_none() {
            write!(self.w, "/>")?;
        } else {
            write!(self.w, ">")?;
        }
        Ok(())
    }

    fn write_end_tag(&mut self, node: Node, element: &Element) -> Result<(), Error> {
        if self.xot.first_child(node).is_some() {
            let fullname = self.fullname_serializer.fullname_or_err(element.name())?;
            write!(self.w, "</{}>", fullname)?;
        }
        self.fullname_serializer
            .pop(&element.namespace_info.to_namespace);
        Ok(())
    }

    fn write_namespace_declaration(
        &mut self,
        node: Node,
        element: &Element,
        prefix_id: PrefixId,
        namespace_id: NamespaceId,
    ) -> Result<(), Error> {
        let namespace = self.xot.namespace_str(namespace_id);
        if prefix_id == self.xot.empty_prefix_id {
            write!(self.w, "xmlns=\"{}\"", namespace)?;
        } else {
            write!(
                self.w,
                "xmlns:{}=\"{}\"",
                self.xot.prefix_str(prefix_id),
                namespace
            )?;
        }
        Ok(())
    }

    fn write_attribute(
        &mut self,
        node: Node,
        element: &Element,
        name_id: NameId,
        value: &str,
    ) -> Result<(), Error> {
        let fullname = self.fullname_serializer.fullname_attr_or_err(name_id)?;
        write!(
            self.w,
            "{}=\"{}\"",
            fullname,
            serialize_attribute(value.into())
        )?;
        Ok(())
    }

    fn write_text(&mut self, node: Node, text: &Text) -> Result<(), Error> {
        write!(self.w, "{}", serialize_text(text.get().into()))?;
        Ok(())
    }

    fn write_comment(&mut self, node: Node, comment: &Comment) -> Result<(), Error> {
        write!(self.w, "<!--{}-->", comment.get())?;
        Ok(())
    }

    fn write_processing_instruction(
        &mut self,
        node: Node,
        processing_instruction: &ProcessingInstruction,
    ) -> Result<(), Error> {
        if let Some(data) = processing_instruction.data() {
            if !data.is_empty() {
                write!(self.w, "<?{} {}?>", processing_instruction.target(), data)?;
            } else {
                write!(self.w, "<?{}?>", processing_instruction.target())?;
            }
        } else {
            write!(self.w, "<?{}?>", processing_instruction.target())?;
        }
        Ok(())
    }

    fn write_space(&mut self) -> Result<(), Error> {
        write!(self.w, " ")?;
        Ok(())
    }
}

fn write_start_tag_open(
    fullname_serializer: &FullnameSerializer,
    element: &Element,
    w: &mut impl Write,
) -> Result<(), Error> {
    write!(
        w,
        "<{}",
        fullname_serializer.fullname_or_err(element.name())?
    )?;
    Ok(())
}

fn write_start_tag_close(xot: &Xot, node: Node, w: &mut impl Write) -> Result<(), Error> {
    if xot.first_child(node).is_none() {
        write!(w, "/>")?;
    } else {
        write!(w, ">")?;
    }
    Ok(())
}

fn write_end_tag(
    xot: &Xot,
    fullname_serializer: &FullnameSerializer,
    node: Node,
    element: &Element,
    w: &mut impl Write,
) -> Result<(), Error> {
    if xot.first_child(node).is_some() {
        let fullname = fullname_serializer.fullname_or_err(element.name())?;
        write!(w, "</{}>", fullname)?;
    }
    Ok(())
}

fn write_namespace_declaration(
    xot: &Xot,
    prefix_id: PrefixId,
    namespace_id: NamespaceId,
    w: &mut impl Write,
) -> Result<(), Error> {
    let namespace = xot.namespace_str(namespace_id);
    if prefix_id == xot.empty_prefix_id {
        write!(w, "xmlns=\"{}\"", namespace)?;
    } else {
        write!(w, "xmlns:{}=\"{}\"", xot.prefix_str(prefix_id), namespace)?;
    }
    Ok(())
}

fn write_attribute(
    fullname_serializer: &FullnameSerializer,
    name_id: NameId,
    value: &str,
    w: &mut impl Write,
) -> Result<(), Error> {
    let fullname = fullname_serializer.fullname_attr_or_err(name_id)?;
    write!(w, "{}=\"{}\"", fullname, serialize_attribute(value.into()))?;
    Ok(())
}

fn write_text(text: &Text, w: &mut impl Write) -> Result<(), Error> {
    write!(w, "{}", serialize_text(text.get().into()))?;
    Ok(())
}

fn write_comment(comment: &Comment, w: &mut impl Write) -> Result<(), Error> {
    write!(w, "<!--{}-->", comment.get())?;
    Ok(())
}

fn write_processing_instruction(
    processing_instruction: &ProcessingInstruction,
    w: &mut impl Write,
) -> Result<(), Error> {
    if let Some(data) = processing_instruction.data() {
        if !data.is_empty() {
            write!(w, "<?{} {}?>", processing_instruction.target(), data)?;
        } else {
            write!(w, "<?{}?>", processing_instruction.target())?;
        }
    } else {
        write!(w, "<?{}?>", processing_instruction.target())?;
    }
    Ok(())
}
