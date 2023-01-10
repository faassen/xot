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
}

impl<'a, W: SerializerWriter> Serializer<'a, W> {
    /// Create a new serializer from a serializer writer.
    pub fn new(xot: &'a Xot<'a>, writer: &'a mut W) -> Self {
        Self { xot, writer }
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
                    self.writer.push_prefixes(&extra_prefixes);
                    // since this will always be at the bottom, it doesn't need to be popped
                }
                self.writer
                    .push_prefixes(&element.namespace_info.to_namespace);
                self.writer.write_start_tag_open(node, element)?;

                // serialize any extra prefixes if this is the top element of
                // a fragment and they aren't declared already
                if node == top_node {
                    for (prefix_id, namespace_id) in self.get_extra_prefixes(node) {
                        if !element.namespace_info.to_namespace.contains_key(&prefix_id) {
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
                    self.writer.write_namespace_declaration(
                        node,
                        element,
                        *prefix_id,
                        *namespace_id,
                    )?;
                }
                self.writer
                    .write_additional_namespace_declarations(node, element)?;

                for (name_id, value) in element.attributes() {
                    self.writer
                        .write_attribute(node, element, *name_id, value)?;
                }
                self.writer.write_additional_attributes(node, element)?;

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
            self.writer.write_end_tag(node, element)?;
            self.writer
                .pop_prefixes(&element.namespace_info.to_namespace);
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
    /// Get the fullname for a name in serialization context
    fn fullname(&self, name_id: NameId) -> Result<String, Error>;
    /// Get the fullname for an attribute in serialiation context
    fn fullname_attr(&self, name_id: NameId) -> Result<String, Error>;
    /// Push new namespace prefixes during serialization
    fn push_prefixes(&mut self, prefixes: &ToNamespace);
    /// Pop namespace prefixes during serialization
    fn pop_prefixes(&mut self, prefixes: &ToNamespace);

    /// Write the start tag opening: e.g `<foo`.
    fn write_start_tag_open(&mut self, node: Node, element: &Element) -> Result<(), Error>;
    /// Write the start tag closing, e.g. `>`, or `/>` if the element is empty.
    fn write_start_tag_close(&mut self, node: Node, element: &Element) -> Result<(), Error>;
    /// Write the end tag, e.g. `</foo>`, or nothing if the element is empty.
    fn write_end_tag(&mut self, node: Node, element: &Element) -> Result<(), Error>;
    /// Write a namespace declaration, e.g. `xmlns:foo="http://example.com"`.
    fn write_namespace_declaration(
        &mut self,
        node: Node,
        element: &Element,
        prefix_id: PrefixId,
        namespace_id: NamespaceId,
    ) -> Result<(), Error>;
    /// Write any additional namespace declarations.
    ///
    /// This can be implemented by a serializer that needs to write additional
    /// namespace declarations after the existing namespace declarations have
    /// been written.
    ///
    /// The default implementation writes nothing.
    fn write_additional_namespace_declarations(
        &mut self,
        _node: Node,
        _element: &Element,
    ) -> Result<(), Error> {
        // by default, do nothing
        Ok(())
    }
    /// Write an attribute, e.g. `foo="bar"`.
    fn write_attribute(
        &mut self,
        node: Node,
        element: &Element,
        name_id: NameId,
        value: &str,
    ) -> Result<(), Error>;
    /// Write any additional attributes.
    ///
    /// This can implemented by a serializer that needs to write additional
    /// attributes after the existing attributes have been written.
    ///
    /// The default implementation writes nothing.
    fn write_additional_attributes(
        &mut self,
        _node: Node,
        _element: &Element,
    ) -> Result<(), Error> {
        // by default, do nothing
        Ok(())
    }
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

    /// Get the fullname for a name in serialization context
    pub fn fullname(&self, name_id: NameId) -> Result<String, Error> {
        self.serializer_writer.fullname(name_id)
    }

    /// Get the fullname for an attribute in serialiation context
    pub fn fullname_attr(&self, name_id: NameId) -> Result<String, Error> {
        self.serializer_writer.fullname_attr(name_id)
    }

    /// Push new namespace prefixes during serialization
    pub fn push_prefixes(&mut self, prefixes: &ToNamespace) {
        self.serializer_writer.push_prefixes(prefixes);
    }

    /// Pop namespace prefixes during serialization
    pub fn pop_prefixes(&mut self, prefixes: &ToNamespace) {
        self.serializer_writer.pop_prefixes(prefixes);
    }

    /// Get opening of start tag
    pub fn get_start_tag_open(&mut self, node: Node, element: &Element) -> Result<String, Error> {
        self.clear();
        self.serializer_writer.write_start_tag_open(node, element)?;
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
    pub fn get_end_tag(&mut self, node: Node, element: &Element) -> Result<String, Error> {
        self.clear();
        self.serializer_writer.write_end_tag(node, element)?;
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
    ) -> Result<String, Error> {
        self.clear();
        self.serializer_writer
            .write_attribute(node, element, name_id, value)?;
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
}

pub(crate) struct XmlSerializerWriter<'a, W: Write> {
    xot: &'a Xot<'a>,
    fullname_serializer: FullnameSerializer<'a>,
    w: W,
}

impl<'a, W: Write> XmlSerializerWriter<'a, W> {
    pub(crate) fn new(xot: &'a Xot<'a>, w: W) -> XmlSerializerWriter<'a, W> {
        let fullname_serializer = FullnameSerializer::new(xot);
        XmlSerializerWriter {
            xot,
            fullname_serializer,
            w,
        }
    }

    pub(crate) fn write(&mut self, s: &str) -> Result<(), Error> {
        self.w.write_all(s.as_bytes())?;
        Ok(())
    }
}

impl<'a, W: Write> SerializerWriter for XmlSerializerWriter<'a, W> {
    fn fullname(&self, name_id: NameId) -> Result<String, Error> {
        self.fullname_serializer
            .fullname_or_err(name_id)
            .map(|s| s.into_owned())
    }

    fn fullname_attr(&self, name_id: NameId) -> Result<String, Error> {
        self.fullname_serializer
            .fullname_attr_or_err(name_id)
            .map(|s| s.into_owned())
    }

    fn push_prefixes(&mut self, prefixes: &ToNamespace) {
        self.fullname_serializer.push(prefixes);
    }

    fn pop_prefixes(&mut self, prefixes: &ToNamespace) {
        self.fullname_serializer.pop(prefixes);
    }

    fn write_start_tag_open(&mut self, _node: Node, _element: &Element) -> Result<(), Error> {
        let fullname = self.fullname(_element.name_id)?;
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

    fn write_end_tag(&mut self, node: Node, _element: &Element) -> Result<(), Error> {
        if self.xot.first_child(node).is_some() {
            let fullname = self.fullname(_element.name_id)?;
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
            write!(self.w, " xmlns=\"{}\"", namespace)?;
        } else {
            write!(
                self.w,
                " xmlns:{}=\"{}\"",
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
        name_id: NameId,
        value: &str,
    ) -> Result<(), Error> {
        let fullname = self.fullname_attr(name_id)?;
        write!(
            self.w,
            " {}=\"{}\"",
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
            fn fullname(&self, name_id: NameId) -> Result<String, Error> {
                self.inner_writer.fullname(name_id)
            }

            fn fullname_attr(&self, name_id: NameId) -> Result<String, Error> {
                self.inner_writer.fullname_attr(name_id)
            }

            fn push_prefixes(&mut self, prefixes: &ToNamespace) {
                self.inner_writer.push_prefixes(prefixes);
            }

            fn pop_prefixes(&mut self, prefixes: &ToNamespace) {
                self.inner_writer.pop_prefixes(prefixes);
            }

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
        }

        let doc = xot.parse(r#"<doc><a/><b/></doc>"#).unwrap();
        let mut writer = StyleWriter::new(&xot);
        xot.serialize_with_writer(doc, &mut writer).unwrap();

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

    #[test]
    fn test_middleware_additional() {
        let mut xot = Xot::new();

        struct Writer<'a, W: Write> {
            xot: &'a Xot<'a>,
            inner_writer: XmlSerializerWriter<'a, W>,
            new_prefix_id: PrefixId,
            new_namespace_id: NamespaceId,
            new_attribute_name_id: NameId,
        }

        impl<'a, W: Write> Writer<'a, W> {
            fn new(
                xot: &'a Xot<'a>,
                w: W,
                new_prefix_id: PrefixId,
                new_namespace_id: NamespaceId,
                new_attribute_name_id: NameId,
            ) -> Writer<'a, W> {
                let inner_writer = XmlSerializerWriter::new(xot, w);
                Writer {
                    inner_writer,
                    xot,
                    new_prefix_id,
                    new_namespace_id,
                    new_attribute_name_id,
                }
            }
        }

        impl<'a, W: Write> SerializerWriter for Writer<'a, W> {
            fn fullname(&self, name_id: NameId) -> Result<String, Error> {
                self.inner_writer.fullname(name_id)
            }

            fn fullname_attr(&self, name_id: NameId) -> Result<String, Error> {
                self.inner_writer.fullname_attr(name_id)
            }

            fn push_prefixes(&mut self, prefixes: &ToNamespace) {
                self.inner_writer.push_prefixes(prefixes);
            }

            fn pop_prefixes(&mut self, prefixes: &ToNamespace) {
                self.inner_writer.pop_prefixes(prefixes);
            }

            fn write_start_tag_open(&mut self, node: Node, element: &Element) -> Result<(), Error> {
                self.inner_writer.write_start_tag_open(node, element)
            }

            fn write_start_tag_close(
                &mut self,
                node: Node,
                element: &Element,
            ) -> Result<(), Error> {
                self.inner_writer.write_start_tag_close(node, element)
            }

            fn write_end_tag(&mut self, node: Node, element: &Element) -> Result<(), Error> {
                self.inner_writer.write_end_tag(node, element)
            }

            fn write_namespace_declaration(
                &mut self,
                node: Node,
                element: &Element,
                prefix_id: PrefixId,
                namespace_id: NamespaceId,
            ) -> Result<(), Error> {
                self.inner_writer.write_namespace_declaration(
                    node,
                    element,
                    prefix_id,
                    namespace_id,
                )
            }

            fn write_attribute(
                &mut self,
                node: Node,
                element: &Element,
                name_id: NameId,
                value: &str,
            ) -> Result<(), Error> {
                self.inner_writer
                    .write_attribute(node, element, name_id, value)
            }

            fn write_text(&mut self, node: Node, text: &Text) -> Result<(), Error> {
                self.inner_writer.write_text(node, text)
            }

            fn write_comment(&mut self, node: Node, comment: &Comment) -> Result<(), Error> {
                self.inner_writer.write_comment(node, comment)
            }

            fn write_processing_instruction(
                &mut self,
                node: Node,
                pi: &ProcessingInstruction,
            ) -> Result<(), Error> {
                self.inner_writer.write_processing_instruction(node, pi)
            }

            fn write_additional_namespace_declarations(
                &mut self,
                node: Node,
                element: &Element,
            ) -> Result<(), Error> {
                self.inner_writer.write_namespace_declaration(
                    node,
                    element,
                    self.new_prefix_id,
                    self.new_namespace_id,
                )?;
                Ok(())
            }

            fn write_additional_attributes(
                &mut self,
                node: Node,
                element: &Element,
            ) -> Result<(), Error> {
                self.inner_writer.write_attribute(
                    node,
                    element,
                    self.new_attribute_name_id,
                    "value",
                )?;
                Ok(())
            }
        }

        let doc = xot
            .parse(r#"<doc><a xmlns:y="http://example.com/y" y="Y"/><b/></doc>"#)
            .unwrap();
        let new_prefix_id = xot.add_prefix("x");
        let new_namespace_id = xot.add_namespace("http://example.com");
        let new_attribute_name_id = xot.add_name("attr");
        let mut buf = Vec::new();
        let mut writer = Writer::new(
            &xot,
            &mut buf,
            new_prefix_id,
            new_namespace_id,
            new_attribute_name_id,
        );
        xot.serialize_with_writer(doc, &mut writer).unwrap();

        let s = String::from_utf8(buf).unwrap();

        assert_eq!(
            s,
            r#"<doc xmlns:x="http://example.com" attr="value"><a xmlns:y="http://example.com/y" xmlns:x="http://example.com" y="Y" attr="value"/><b xmlns:x="http://example.com" attr="value"/></doc>"#
        );
    }
}
