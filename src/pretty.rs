use std::io::Write;

use crate::error::Error;
use crate::name::NameId;
use crate::namespace::NamespaceId;
use crate::prefix::PrefixId;
use crate::serializer::{SerializerWriter, XmlSerializerWriter};
use crate::xmlvalue::{Comment, Element, ProcessingInstruction, Text, ToNamespace, ValueType};
use crate::xotdata::{Node, Xot};

// pretty printing

// approach: modify tree to insert whitespace
// alternative approach: modify serializer to insert whitespace

// modify serializer rule:
// when we insert a tag open, we put in the indentation level,
// unless this tag is part of a mixed content node
// when we insert a tag close, we reduce the indentation level
// each tag is inserted. If we insert any other node, we don't mess with indentation.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StackEntry {
    Unmixed,
    Mixed,
}

struct PrettyWriter<'a, W: Write> {
    xot: &'a Xot<'a>,
    inner_writer: XmlSerializerWriter<'a, W>,
    stack: Vec<StackEntry>,
}

impl<'a, W: Write> PrettyWriter<'a, W> {
    fn new(xot: &'a Xot<'a>, w: W) -> PrettyWriter<'a, W> {
        let inner_writer = XmlSerializerWriter::new(xot, w);
        PrettyWriter {
            xot,
            inner_writer,
            stack: Vec::new(),
        }
    }

    fn unmixed(&mut self) {
        self.stack.push(StackEntry::Unmixed);
    }

    fn mixed(&mut self) {
        self.stack.push(StackEntry::Mixed);
    }

    fn in_mixed(&self) -> bool {
        self.stack.iter().any(|e| *e == StackEntry::Mixed)
    }

    fn pop(&mut self) {
        self.stack.pop();
    }

    fn write_indentation(&mut self) -> Result<(), Error> {
        if self.in_mixed() {
            return Ok(());
        }
        let indent = self
            .stack
            .iter()
            .filter(|e| **e == StackEntry::Unmixed)
            .count();
        let indent = " ".repeat(indent * 2);
        self.inner_writer.write(&indent)?;
        Ok(())
    }

    fn write_newline(&mut self) -> Result<(), Error> {
        self.inner_writer.write("\n")?;
        Ok(())
    }

    fn indent(&mut self, node: Node) -> Result<(), Error> {
        let has_children = self.xot.first_child(node).is_some();
        if has_children {
            if !self.has_text_child(node) {
                self.write_newline()?;
                self.unmixed();
            } else {
                self.mixed();
            }
        }
        Ok(())
    }

    fn dedent(&mut self, node: Node) -> Result<(), Error> {
        let has_children = self.xot.first_child(node).is_some();
        if has_children {
            let was_in_mixed = self.in_mixed();
            self.pop();
            if !was_in_mixed {
                self.write_indentation()?;
            }
        }
        Ok(())
    }

    fn dedent_newline(&mut self) -> Result<(), Error> {
        if !self.in_mixed() {
            self.write_newline()?;
        }
        Ok(())
    }

    fn has_text_child(&self, node: Node) -> bool {
        self.xot
            .children(node)
            .any(|child| self.xot.value_type(child) == ValueType::Text)
    }
}

impl<'a, W: Write> SerializerWriter for PrettyWriter<'a, W> {
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
        self.write_indentation()?;
        self.inner_writer.write_start_tag_open(node, element)
    }

    fn write_start_tag_close(&mut self, node: Node, element: &Element) -> Result<(), Error> {
        self.inner_writer.write_start_tag_close(node, element)?;

        self.indent(node)?;
        Ok(())
    }

    fn write_end_tag(&mut self, node: Node, element: &Element) -> Result<(), Error> {
        self.dedent(node)?;
        self.inner_writer.write_end_tag(node, element)?;
        self.dedent_newline()?;
        Ok(())
    }

    fn write_namespace_declaration(
        &mut self,
        node: Node,
        element: &Element,
        prefix_id: PrefixId,
        namespace_id: NamespaceId,
    ) -> Result<(), Error> {
        self.inner_writer
            .write_namespace_declaration(node, element, prefix_id, namespace_id)?;
        Ok(())
    }

    fn write_attribute(
        &mut self,
        node: Node,
        element: &Element,
        name_id: NameId,
        value: &str,
    ) -> Result<(), Error> {
        self.inner_writer
            .write_attribute(node, element, name_id, value)?;
        Ok(())
    }

    fn write_text(&mut self, node: Node, text: &Text) -> Result<(), Error> {
        self.inner_writer.write_text(node, text)?;
        Ok(())
    }

    fn write_comment(&mut self, node: Node, comment: &Comment) -> Result<(), Error> {
        self.write_indentation()?;
        self.inner_writer.write_comment(node, comment)?;
        self.dedent_newline()?;
        Ok(())
    }

    fn write_processing_instruction(
        &mut self,
        node: Node,
        pi: &ProcessingInstruction,
    ) -> Result<(), Error> {
        self.write_indentation()?;
        self.inner_writer.write_processing_instruction(node, pi)?;
        self.dedent_newline()?;
        Ok(())
    }

    fn write_space(&mut self) -> Result<(), Error> {
        self.inner_writer.write_space()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn test_only_elements() {
        let mut xot = Xot::new();

        let doc = xot.parse(r#"<doc><a><b/></a></doc>"#).unwrap();

        let mut buf = Vec::new();
        let mut writer = PrettyWriter::new(&xot, &mut buf);
        xot.serialize_with_writer(doc, &mut writer).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert_snapshot!(s);
    }

    #[test]
    fn test_only_elements_multi() {
        let mut xot = Xot::new();

        let doc = xot
            .parse(r#"<doc><a><b/></a><a><b/><b/></a></doc>"#)
            .unwrap();

        let mut buf = Vec::new();
        let mut writer = PrettyWriter::new(&xot, &mut buf);
        xot.serialize_with_writer(doc, &mut writer).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert_snapshot!(s);
    }

    #[test]
    fn test_text() {
        let mut xot = Xot::new();

        let doc = xot.parse(r#"<doc><a>text</a><a>text 2</a></doc>"#).unwrap();

        let mut buf = Vec::new();
        let mut writer = PrettyWriter::new(&xot, &mut buf);
        xot.serialize_with_writer(doc, &mut writer).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert_snapshot!(s);
    }

    #[test]
    fn test_comment() {
        let mut xot = Xot::new();

        let doc = xot
            .parse(r#"<doc><a><!--hello--><!--world--></a></doc>"#)
            .unwrap();

        let mut buf = Vec::new();
        let mut writer = PrettyWriter::new(&xot, &mut buf);
        xot.serialize_with_writer(doc, &mut writer).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert_snapshot!(s);
    }

    #[test]
    fn test_pi() {
        let mut xot = Xot::new();
        let doc = xot.parse(r#"<doc><a><?pi?><?pi?></a></doc>"#).unwrap();
        let mut buf = Vec::new();
        let mut writer = PrettyWriter::new(&xot, &mut buf);
        xot.serialize_with_writer(doc, &mut writer).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert_snapshot!(s);
    }
}
