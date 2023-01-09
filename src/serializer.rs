use std::io::Write;

use crate::access::NodeEdge;
use crate::entity::{serialize_attribute, serialize_text};
use crate::error::Error;
use crate::fullname::FullnameSerializer;
use crate::name::NameId;
use crate::namespace::NamespaceId;
use crate::prefix::PrefixId;
use crate::xmlvalue::{
    Comment, Element, ProcessingInstruction, Text, ToNamespace, Value, ValueType,
};
use crate::xotdata::{Node, Xot};

pub(crate) struct Serializer<'a, W: SerializerWriter> {
    xot: &'a Xot<'a>,
    writer: &'a mut W,
    fullname_serializer: FullnameSerializer<'a>,
}

impl<'a, W: SerializerWriter> Serializer<'a, W> {
    /// Create a new serializer from a serializer writer.

    pub fn new(xot: &'a Xot<'a>, writer: &'a mut W) -> Self {
        let fullname_serializer = FullnameSerializer::new(xot);
        Self {
            xot,
            writer,
            fullname_serializer,
        }
    }

    fn get_extra_prefixes(&self, node: Node) -> ToNamespace {
        // collect namespace prefixes for all ancestors of the fragment
        if let Some(parent) = self.xot.parent(node) {
            if self.xot.value_type(parent) != ValueType::Root {
                self.xot.to_namespace_in_scope(parent)
            } else {
                ToNamespace::new()
            }
        } else {
            ToNamespace::new()
        }
    }

    /// Serialize a node and all its children using the writer.
    ///
    /// The writer controls what happens with the serialized data.
    pub(crate) fn serialize_node(&mut self, node: Node) -> Result<(), Error> {
        for edge in self.xot.traverse(node) {
            match edge {
                NodeEdge::Start(current_node) => {
                    self.handle_edge_start(node, current_node)?;
                }
                NodeEdge::End(current_node) => {
                    self.handle_edge_end(current_node)?;
                }
            }
        }
        Ok(())
    }

    fn handle_edge_start(&mut self, top_node: Node, node: Node) -> Result<(), Error> {
        let value = self.xot.value(node);
        match value {
            Value::Root => {}
            Value::Element(element) => {
                if node == top_node {
                    let extra_prefixes = self.get_extra_prefixes(node);
                    self.fullname_serializer.push(&extra_prefixes);
                    // since this will always be at the bottom, it doesn't need to be popped
                }
                self.fullname_serializer
                    .push(&element.namespace_info.to_namespace);
                let fullname = self.fullname_serializer.fullname_or_err(element.name_id)?;
                self.writer
                    .write_start_tag_open(node, element, fullname.as_ref())?;

                // serialize any extra prefixes if this is the top element of
                // a fragment and they aren't declared already
                if node == top_node {
                    for (prefix_id, namespace_id) in self.get_extra_prefixes(node) {
                        if !element.namespace_info.to_namespace.contains_key(&prefix_id) {
                            self.writer.write_space()?;
                            self.writer.write_namespace_declaration(
                                node,
                                element,
                                prefix_id,
                                namespace_id,
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
                    let fullname = self.fullname_serializer.fullname_attr_or_err(*name_id)?;
                    self.writer.write_attribute(
                        node,
                        element,
                        *name_id,
                        value,
                        fullname.as_ref(),
                    )?;
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

    fn handle_edge_end(&mut self, node: Node) -> Result<(), Error> {
        let value = self.xot.value(node);
        if let Value::Element(element) = value {
            let fullname = self.fullname_serializer.fullname_or_err(element.name_id)?;
            self.writer
                .write_end_tag(node, element, fullname.as_ref())?;
            self.fullname_serializer
                .pop(&element.namespace_info.to_namespace);
        }
        Ok(())
    }
}

/// A trait for writing XML.
///
/// The serializer invokes methods on this trait to actually write the XML.
///
/// This is an advanced trait that can be used with
/// [`Xot::serialize_with_writer`], [`Xot::serialize_node_with_writer`] and
/// [`Xot::serialize_or_missing_prefix_with_writer`].
///
/// Usually you'd implement this trait in your own struct by using
/// [`xot::StringWriter`](`crate::serializer::StringWriter`), which does the
/// standard XML serialization. The default serialization process can be
/// understood as using
/// [`xot::StringWriter`](`crate::serializer::StringWriter`) to implement each
/// of these methods and pass the resulting strings off to an output buffer.
/// But in your implementation of this trait you can decide to output
/// additional content, or to filter out particular strings and not write them
/// at all.
pub trait SerializerWriter {
    /// Write the start tag opening: e.g `<foo`.
    fn write_start_tag_open(
        &mut self,
        node: Node,
        element: &Element,
        fullname: &str,
    ) -> Result<(), Error>;
    /// Write the start tag closing, e.g. `>`, or `/>` if the element is empty.
    fn write_start_tag_close(&mut self, node: Node, element: &Element) -> Result<(), Error>;
    /// Write the end tag, e.g. `</foo>`, or nothing if the element is empty.
    fn write_end_tag(&mut self, node: Node, element: &Element, fullname: &str)
        -> Result<(), Error>;
    /// Write a namespace declaration, e.g. `xmlns:foo="http://example.com"`.
    fn write_namespace_declaration(
        &mut self,
        node: Node,
        element: &Element,
        prefix_id: PrefixId,
        namespace_id: NamespaceId,
    ) -> Result<(), Error>;
    /// Write an attribute, e.g. `foo="bar"`.
    fn write_attribute(
        &mut self,
        node: Node,
        element: &Element,
        name_id: NameId,
        value: &str,
        fullname: &str,
    ) -> Result<(), Error>;
    /// Write text, e.g `foo`.
    fn write_text(&mut self, node: Node, text: &Text) -> Result<(), Error>;
    /// Write a comment, e.g. `<!-- foo -->`.
    fn write_comment(&mut self, node: Node, comment: &Comment) -> Result<(), Error>;
    /// Write a processing instruction, e.g. `<?foo bar?>`.
    fn write_processing_instruction(
        &mut self,
        node: Node,
        pi: &ProcessingInstruction,
    ) -> Result<(), Error>;
    /// Write a space, e.g. ` `.
    fn write_space(&mut self) -> Result<(), Error>;
}

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

/// Return strings for written XML.
///
/// This is like a SerializerWriter, but returns strings. This is useful if you
/// want to create your own custom
/// [`xot::SerializerWriter`](`crate::serializer::SerializerWriter`) trait
/// implementation and use this inside it. You can decide whether to write a
/// string at all, and also output additional content.
pub struct StringWriter<'a> {
    serializer_writer: XmlSerializerWriter<'a, LastPushedWrite>,
}

impl<'a> StringWriter<'a> {
    /// Create a new StringWriter.
    pub fn new(xot: &'a Xot) -> StringWriter<'a> {
        StringWriter {
            serializer_writer: XmlSerializerWriter::new(xot, LastPushedWrite::new()),
        }
    }

    fn clear(&mut self) {
        self.serializer_writer.w.clear();
    }

    fn get(&self) -> String {
        self.serializer_writer.w.get()
    }

    /// Get opening of start tag
    pub fn get_start_tag_open(
        &mut self,
        node: Node,
        element: &Element,
        fullname: &str,
    ) -> Result<String, Error> {
        self.clear();
        self.serializer_writer
            .write_start_tag_open(node, element, fullname)?;
        Ok(self.get())
    }
    /// Get end of start tag
    pub fn get_start_tag_close(&mut self, node: Node, element: &Element) -> Result<String, Error> {
        self.clear();
        self.serializer_writer
            .write_start_tag_close(node, element)?;
        Ok(self.get())
    }
    /// Get end tag
    pub fn get_end_tag(
        &mut self,
        node: Node,
        element: &Element,
        fullname: &str,
    ) -> Result<String, Error> {
        self.clear();
        self.serializer_writer
            .write_end_tag(node, element, fullname)?;
        Ok(self.get())
    }
    /// Get namespace declaration
    pub fn get_namespace_declaration(
        &mut self,
        node: Node,
        element: &Element,
        prefix_id: PrefixId,
        namespace_id: NamespaceId,
    ) -> Result<String, Error> {
        self.clear();
        self.serializer_writer.write_namespace_declaration(
            node,
            element,
            prefix_id,
            namespace_id,
        )?;
        Ok(self.get())
    }
    /// Get attribute
    pub fn get_attribute(
        &mut self,
        node: Node,
        element: &Element,
        name_id: NameId,
        value: &str,
        fullname: &str,
    ) -> Result<String, Error> {
        self.clear();
        self.serializer_writer
            .write_attribute(node, element, name_id, value, fullname)?;
        Ok(self.get())
    }
    /// Get text
    pub fn get_text(&mut self, node: Node, text: &Text) -> Result<String, Error> {
        self.clear();
        self.serializer_writer.write_text(node, text)?;
        Ok(self.get())
    }
    /// Get comment
    pub fn get_comment(&mut self, node: Node, comment: &Comment) -> Result<String, Error> {
        self.clear();
        self.serializer_writer.write_comment(node, comment)?;
        Ok(self.get())
    }
    /// Get processing instruction
    pub fn get_processing_instruction(
        &mut self,
        node: Node,
        pi: &ProcessingInstruction,
    ) -> Result<String, Error> {
        self.clear();
        self.serializer_writer
            .write_processing_instruction(node, pi)?;
        Ok(self.get())
    }
    /// Get space
    pub fn get_space(&mut self) -> Result<String, Error> {
        Ok(" ".to_string())
    }
}

pub(crate) struct XmlSerializerWriter<'a, W: Write> {
    xot: &'a Xot<'a>,
    w: W,
}

impl<'a, W: Write> XmlSerializerWriter<'a, W> {
    pub(crate) fn new(xot: &'a Xot<'a>, w: W) -> XmlSerializerWriter<'a, W> {
        XmlSerializerWriter { xot, w }
    }
}

impl<'a, W: Write> SerializerWriter for XmlSerializerWriter<'a, W> {
    fn write_start_tag_open(
        &mut self,
        _node: Node,
        _element: &Element,
        fullname: &str,
    ) -> Result<(), Error> {
        write!(self.w, "<{}", fullname)?;
        Ok(())
    }

    fn write_start_tag_close(&mut self, node: Node, _element: &Element) -> Result<(), Error> {
        if self.xot.first_child(node).is_none() {
            write!(self.w, "/>")?;
        } else {
            write!(self.w, ">")?;
        }
        Ok(())
    }

    fn write_end_tag(
        &mut self,
        node: Node,
        _element: &Element,
        fullname: &str,
    ) -> Result<(), Error> {
        if self.xot.first_child(node).is_some() {
            write!(self.w, "</{}>", fullname)?;
        }
        Ok(())
    }

    fn write_namespace_declaration(
        &mut self,
        _node: Node,
        _element: &Element,
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
        _node: Node,
        _element: &Element,
        _name_id: NameId,
        value: &str,
        fullname: &str,
    ) -> Result<(), Error> {
        write!(
            self.w,
            "{}=\"{}\"",
            fullname,
            serialize_attribute(value.into())
        )?;
        Ok(())
    }

    fn write_text(&mut self, _node: Node, text: &Text) -> Result<(), Error> {
        write!(self.w, "{}", serialize_text(text.get().into()))?;
        Ok(())
    }

    fn write_comment(&mut self, _node: Node, comment: &Comment) -> Result<(), Error> {
        write!(self.w, "<!--{}-->", comment.get())?;
        Ok(())
    }

    fn write_processing_instruction(
        &mut self,
        _node: Node,
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
        let mut writer = XmlSerializerWriter::new(&xot, &mut w);
        let mut serializer = Serializer::new(&xot, &mut writer);
        serializer.serialize_node(doc).unwrap();
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

        struct StyleWriter<'a> {
            data: Vec<StyledText>,
            inner_writer: StringWriter<'a>,
            xot: &'a Xot<'a>,
        }

        impl<'a> StyleWriter<'a> {
            fn new(xot: &'a Xot<'a>) -> StyleWriter<'a> {
                let inner_writer = StringWriter::new(xot);
                StyleWriter {
                    data: Vec::new(),
                    inner_writer,
                    xot,
                }
            }
        }

        impl<'a> SerializerWriter for StyleWriter<'a> {
            fn write_start_tag_open(
                &mut self,
                node: Node,
                element: &Element,
                fullname: &str,
            ) -> Result<(), Error> {
                let text = self
                    .inner_writer
                    .get_start_tag_open(node, element, fullname)?;
                let name_a = self.xot.name("a").unwrap();
                if element.name() == name_a {
                    self.data.push(StyledText::StyleStart);
                    self.data.push(StyledText::Text(text));
                } else {
                    self.data.push(StyledText::Text(text));
                }
                Ok(())
            }

            fn write_start_tag_close(
                &mut self,
                node: Node,
                element: &Element,
            ) -> Result<(), Error> {
                let text = self.inner_writer.get_start_tag_close(node, element)?;
                self.data.push(StyledText::Text(text));
                Ok(())
            }

            fn write_end_tag(
                &mut self,
                node: Node,
                element: &Element,
                fullname: &str,
            ) -> Result<(), Error> {
                let text = self.inner_writer.get_end_tag(node, element, fullname)?;
                let name_a = self.xot.name("a").unwrap();
                if element.name() == name_a {
                    self.data.push(StyledText::Text(text));
                    self.data.push(StyledText::StyleEnd);
                } else {
                    self.data.push(StyledText::Text(text));
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
                let text = self.inner_writer.get_namespace_declaration(
                    node,
                    element,
                    prefix_id,
                    namespace_id,
                )?;
                self.data.push(StyledText::Text(text));
                Ok(())
            }

            fn write_attribute(
                &mut self,
                node: Node,
                element: &Element,
                name_id: NameId,
                value: &str,
                fullname: &str,
            ) -> Result<(), Error> {
                let text = self
                    .inner_writer
                    .get_attribute(node, element, name_id, value, fullname)?;
                self.data.push(StyledText::Text(text));
                Ok(())
            }

            fn write_text(&mut self, node: Node, text: &Text) -> Result<(), Error> {
                let text = self.inner_writer.get_text(node, text)?;
                self.data.push(StyledText::Text(text));
                Ok(())
            }

            fn write_comment(&mut self, node: Node, comment: &Comment) -> Result<(), Error> {
                let text = self.inner_writer.get_comment(node, comment)?;
                self.data.push(StyledText::Text(text));
                Ok(())
            }

            fn write_processing_instruction(
                &mut self,
                node: Node,
                pi: &ProcessingInstruction,
            ) -> Result<(), Error> {
                let text = self.inner_writer.get_processing_instruction(node, pi)?;
                self.data.push(StyledText::Text(text));
                Ok(())
            }

            fn write_space(&mut self) -> Result<(), Error> {
                let text = self.inner_writer.get_space()?;
                self.data.push(StyledText::Text(text));
                Ok(())
            }
        }

        let mut writer = StyleWriter::new(&xot);
        let mut serializer = Serializer::new(&xot, &mut writer);
        serializer.serialize_node(doc).unwrap();

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
