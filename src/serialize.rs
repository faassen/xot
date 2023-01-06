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

use crate::name::NameId;
use crate::xmlvalue::{Comment, ProcessingInstruction, Text};

struct Serializer<'a, W: SerializerWriter> {
    writer: &'a mut W,
    extra_prefixes: ToNamespace,
}

impl<'a, W: SerializerWriter> Serializer<'a, W> {
    fn new(writer: &'a mut W, extra_prefixes: ToNamespace) -> Self {
        Self {
            writer,
            extra_prefixes,
        }
    }

    fn serialize_node(&mut self, xot: &Xot, node: Node) -> Result<(), Error> {
        for edge in xot.traverse(node) {
            match edge {
                NodeEdge::Start(current_node) => {
                    self.handle_edge_start(xot, node, current_node)?;
                }
                NodeEdge::End(current_node) => {
                    self.handle_edge_end(xot, current_node)?;
                }
            }
        }
        Ok(())
    }

    fn handle_edge_start(&mut self, xot: &Xot, top_node: Node, node: Node) -> Result<(), Error> {
        let value = xot.value(node);
        match value {
            Value::Root => {}
            Value::Element(element) => {
                self.writer.write_start_tag_open(node, element)?;

                // serialize any extra prefixes if this is the top element of
                // a fragment and they aren't declared already
                if node == top_node {
                    for (prefix_id, namespace_id) in &self.extra_prefixes {
                        if !element.namespace_info.to_namespace.contains_key(prefix_id) {
                            self.writer.write_space()?;
                            self.writer.write_namespace_declaration(
                                node,
                                element,
                                *prefix_id,
                                *namespace_id,
                            )?;
                        }
                    }
                }

                for (prefix_id, namespace_id) in element.prefixes() {
                    self.writer.write_space()?;
                    self.writer.write_namespace_declaration(
                        node,
                        element,
                        *prefix_id,
                        *namespace_id,
                    )?;
                }

                for (name_id, value) in element.attributes() {
                    self.writer.write_space()?;
                    self.writer
                        .write_attribute(node, element, *name_id, value)?;
                }

                self.writer.write_start_tag_close(node, element)?;
            }
            Value::Text(text) => {
                self.writer.write_text(node, text)?;
            }
            Value::Comment(comment) => {
                self.writer.write_comment(node, comment)?;
            }
            Value::ProcessingInstruction(pi) => {
                self.writer.write_processing_instruction(node, pi)?;
            }
        }
        Ok(())
    }

    fn handle_edge_end(&mut self, xot: &Xot, node: Node) -> Result<(), Error> {
        let value = xot.value(node);
        if let Value::Element(element) = value {
            self.writer.write_end_tag(node, element)?;
        }
        Ok(())
    }
}

pub trait SerializerWriter {
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
    fullname_serializer: FullnameSerializer<'a>,
    w: W,
}

impl<'a, W: Write> XmlSerializerWriter<'a, W> {
    pub(crate) fn new(
        xot: &'a Xot<'a>,
        w: W,
        extra_prefixes: &ToNamespace,
    ) -> XmlSerializerWriter<'a, W> {
        let fullname_serializer =
            FullnameSerializer::with_to_namespace(extra_prefixes.clone(), xot);
        XmlSerializerWriter {
            xot,
            fullname_serializer,
            w,
        }
    }
}

impl<'a, W: Write> SerializerWriter for XmlSerializerWriter<'a, W> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_writer_record() {
        let mut xot = Xot::new();
        let doc = xot.parse(r#"<doc><a/></doc>"#).unwrap();

        // this is a quoting write, which writes to a document, thus quoting
        // the original XML
        struct RecordingWrite {
            data: Vec<String>,
        }

        impl RecordingWrite {
            fn new() -> RecordingWrite {
                RecordingWrite { data: Vec::new() }
            }
        }

        impl Write for RecordingWrite {
            fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
                self.data.push(String::from_utf8(buf.to_vec()).unwrap());
                Ok(buf.len())
            }

            fn flush(&mut self) -> Result<(), std::io::Error> {
                Ok(())
            }
        }

        let mut w = RecordingWrite::new();
        let extra_prefixes = ToNamespace::new();
        let mut writer = XmlSerializerWriter::new(&xot, &mut w, &extra_prefixes);
        let mut serializer = Serializer::new(&mut writer, extra_prefixes);
        serializer.serialize_node(&xot, doc).unwrap();
        assert_eq!(w.data.join(""), r#"<doc><a/></doc>"#);
    }

    #[test]
    fn test_middleware() {
        let mut xot = Xot::new();

        #[derive(Debug, PartialEq)]
        enum StyledText {
            Text(String),
            StyleStart,
            StyleEnd,
        }
        let doc = xot.parse(r#"<doc><a/><b/></doc>"#).unwrap();

        struct LastPushedWrite {
            last_pushed: Vec<String>,
        }

        impl LastPushedWrite {
            fn new() -> LastPushedWrite {
                LastPushedWrite {
                    last_pushed: Vec::new(),
                }
            }

            fn clear(&mut self) {
                self.last_pushed.clear();
            }

            fn get(&self) -> String {
                self.last_pushed.join("")
            }
        }

        impl Write for LastPushedWrite {
            fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
                self.last_pushed
                    .push(String::from_utf8(buf.to_vec()).unwrap());
                Ok(buf.len())
            }

            fn flush(&mut self) -> Result<(), std::io::Error> {
                Ok(())
            }
        }

        struct StyleWriter<'a> {
            data: Vec<StyledText>,
            inner_writer: XmlSerializerWriter<'a, LastPushedWrite>,
            xot: &'a Xot<'a>,
        }

        impl<'a> StyleWriter<'a> {
            fn new(xot: &'a Xot<'a>, extra_prefixes: &ToNamespace) -> StyleWriter<'a> {
                let last_pushed = LastPushedWrite::new();
                let inner_writer = XmlSerializerWriter::new(&xot, last_pushed, &extra_prefixes);
                StyleWriter {
                    data: Vec::new(),
                    inner_writer,
                    xot,
                }
            }

            fn clear(&mut self) {
                self.inner_writer.w.clear();
            }

            fn get(&self) -> String {
                self.inner_writer.w.get()
            }
        }

        impl<'a> SerializerWriter for StyleWriter<'a> {
            fn write_start_tag_open(&mut self, node: Node, element: &Element) -> Result<(), Error> {
                self.clear();
                self.inner_writer.write_start_tag_open(node, element)?;
                let name_a = self.xot.name("a").unwrap();
                if element.name() == name_a {
                    self.data.push(StyledText::StyleStart);
                    self.data.push(StyledText::Text(self.get()));
                } else {
                    self.data.push(StyledText::Text(self.get()));
                }
                Ok(())
            }

            fn write_start_tag_close(
                &mut self,
                node: Node,
                element: &Element,
            ) -> Result<(), Error> {
                self.clear();
                self.inner_writer.write_start_tag_close(node, element)?;
                self.data.push(StyledText::Text(self.get()));
                Ok(())
            }

            fn write_end_tag(&mut self, node: Node, element: &Element) -> Result<(), Error> {
                self.clear();
                self.inner_writer.write_end_tag(node, element)?;
                let name_a = self.xot.name("a").unwrap();
                if element.name() == name_a {
                    self.data.push(StyledText::Text(self.get()));
                    self.data.push(StyledText::StyleEnd);
                } else {
                    self.data.push(StyledText::Text(self.get()));
                }
                Ok(())
            }

            fn write_namespace_declaration(
                &mut self,
                node: Node,
                element: &Element,
                prefix_id: PrefixId,
                namespace_id: NamespaceId,
            ) -> Result<(), Error> {
                self.clear();
                self.inner_writer.write_namespace_declaration(
                    node,
                    element,
                    prefix_id,
                    namespace_id,
                )?;
                self.data.push(StyledText::Text(self.get()));
                Ok(())
            }

            fn write_attribute(
                &mut self,
                node: Node,
                element: &Element,
                name_id: NameId,
                value: &str,
            ) -> Result<(), Error> {
                self.clear();
                self.inner_writer
                    .write_attribute(node, element, name_id, value)?;
                self.data.push(StyledText::Text(self.get()));
                Ok(())
            }

            fn write_text(&mut self, node: Node, text: &Text) -> Result<(), Error> {
                self.clear();
                self.inner_writer.write_text(node, text)?;
                self.data.push(StyledText::Text(self.get()));
                Ok(())
            }

            fn write_comment(&mut self, node: Node, comment: &Comment) -> Result<(), Error> {
                self.clear();
                self.inner_writer.write_comment(node, comment)?;
                self.data.push(StyledText::Text(self.get()));
                Ok(())
            }

            fn write_processing_instruction(
                &mut self,
                node: Node,
                pi: &ProcessingInstruction,
            ) -> Result<(), Error> {
                self.clear();
                self.inner_writer.write_processing_instruction(node, pi)?;
                self.data.push(StyledText::Text(self.get()));
                Ok(())
            }

            fn write_space(&mut self) -> Result<(), Error> {
                self.clear();
                self.inner_writer.write_space()?;
                self.data.push(StyledText::Text(self.get()));
                Ok(())
            }
        }

        let extra_prefixes = ToNamespace::new();

        let mut writer = StyleWriter::new(&xot, &extra_prefixes);
        let mut serializer = Serializer::new(&mut writer, extra_prefixes);
        serializer.serialize_node(&xot, doc).unwrap();

        let data = &writer.data;

        assert_eq!(
            data,
            &vec![
                StyledText::Text("<doc".to_string()),
                StyledText::Text(">".to_string()),
                StyledText::StyleStart,
                StyledText::Text("<a".to_string()),
                StyledText::Text("/>".to_string()),
                StyledText::Text("".to_string()),
                StyledText::StyleEnd,
                StyledText::Text("<b".to_string()),
                StyledText::Text("/>".to_string()),
                StyledText::Text("".to_string()),
                StyledText::Text("</doc>".to_string())
            ]
        );
    }
}
