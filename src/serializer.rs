use std::io::Write;

use crate::access::NodeEdge;
use crate::entity::{serialize_attribute, serialize_text};
use crate::error::Error;
use crate::fullname::FullnameSerializer;
use crate::name::NameId;
use crate::namespace::NamespaceId;
use crate::prefix::PrefixId;
use crate::xmlvalue::{Comment, Element, ProcessingInstruction, Text, ToNamespace, Value};
use crate::xotdata::{Node, Xot};

pub(crate) struct Serializer<'a, W: SerializerWriter> {
    writer: &'a mut W,
    extra_prefixes: ToNamespace,
}

impl<'a, W: SerializerWriter> Serializer<'a, W> {
    pub(crate) fn new(writer: &'a mut W, extra_prefixes: ToNamespace) -> Self {
        Self {
            writer,
            extra_prefixes,
        }
    }

    pub(crate) fn serialize_node(&mut self, xot: &Xot, node: Node) -> Result<(), Error> {
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

struct StringWriter<'a> {
    serializer_writer: XmlSerializerWriter<'a, LastPushedWrite>,
}

impl<'a> StringWriter<'a> {
    fn new(xot: &'a Xot, extra_prefixes: &ToNamespace) -> StringWriter<'a> {
        StringWriter {
            serializer_writer: XmlSerializerWriter::new(
                xot,
                LastPushedWrite::new(),
                extra_prefixes,
            ),
        }
    }

    fn clear(&mut self) {
        self.serializer_writer.w.clear();
    }

    fn get(&self) -> String {
        self.serializer_writer.w.get()
    }

    fn get_start_tag_open(&mut self, node: Node, element: &Element) -> Result<String, Error> {
        self.clear();
        self.serializer_writer.write_start_tag_open(node, element)?;
        Ok(self.get())
    }
    fn get_start_tag_close(&mut self, node: Node, element: &Element) -> Result<String, Error> {
        self.clear();
        self.serializer_writer
            .write_start_tag_close(node, element)?;
        Ok(self.get())
    }
    fn get_end_tag(&mut self, node: Node, element: &Element) -> Result<String, Error> {
        self.clear();
        self.serializer_writer.write_end_tag(node, element)?;
        Ok(self.get())
    }
    fn get_namespace_declaration(
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
    fn get_attribute(
        &mut self,
        node: Node,
        element: &Element,
        name_id: NameId,
        value: &str,
    ) -> Result<String, Error> {
        self.clear();
        self.serializer_writer
            .write_attribute(node, element, name_id, value)?;
        Ok(self.get())
    }
    fn get_text(&mut self, node: Node, text: &Text) -> Result<String, Error> {
        self.clear();
        self.serializer_writer.write_text(node, text)?;
        Ok(self.get())
    }
    fn get_comment(&mut self, node: Node, comment: &Comment) -> Result<String, Error> {
        self.clear();
        self.serializer_writer.write_comment(node, comment)?;
        Ok(self.get())
    }
    fn get_processing_instruction(
        &mut self,
        node: Node,
        pi: &ProcessingInstruction,
    ) -> Result<String, Error> {
        self.clear();
        self.serializer_writer
            .write_processing_instruction(node, pi)?;
        Ok(self.get())
    }
    fn get_space(&mut self) -> Result<String, Error> {
        Ok(" ".to_string())
    }
}

pub(crate) struct XmlSerializerWriter<'a, W: Write> {
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

        struct StyleWriter<'a> {
            data: Vec<StyledText>,
            inner_writer: StringWriter<'a>,
            xot: &'a Xot<'a>,
        }

        impl<'a> StyleWriter<'a> {
            fn new(xot: &'a Xot<'a>, extra_prefixes: &ToNamespace) -> StyleWriter<'a> {
                let inner_writer = StringWriter::new(xot, extra_prefixes);
                StyleWriter {
                    data: Vec::new(),
                    inner_writer,
                    xot,
                }
            }
        }

        impl<'a> SerializerWriter for StyleWriter<'a> {
            fn write_start_tag_open(&mut self, node: Node, element: &Element) -> Result<(), Error> {
                let text = self.inner_writer.get_start_tag_open(node, element)?;
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

            fn write_end_tag(&mut self, node: Node, element: &Element) -> Result<(), Error> {
                let text = self.inner_writer.get_end_tag(node, element)?;
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
            ) -> Result<(), Error> {
                let text = self
                    .inner_writer
                    .get_attribute(node, element, name_id, value)?;
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
