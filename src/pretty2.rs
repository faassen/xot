use std::io;
use std::sync::Arc;

use crate::error::Error;
use crate::name::NameId;
use crate::namespace::NamespaceId;
use crate::prefix::PrefixId;
use crate::serializer2::ToBeSerializedIterator;
use crate::serializer2::{serialize_node, SerializationData, ToBeSerialized, XmlSerializer};
use crate::xmlvalue::{Comment, Element, ProcessingInstruction, Text, ToNamespace, ValueType};
use crate::xotdata::{Node, Xot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StackEntry {
    Unmixed,
    Mixed,
}

pub(crate) struct PrettySerializer<'a> {
    xot: &'a Xot<'a>,
    stack: Vec<StackEntry>,
}

impl<'a> PrettySerializer<'a> {
    pub(crate) fn new(xot: &'a Xot<'a>) -> PrettySerializer<'a> {
        PrettySerializer {
            xot,
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

    fn get_indentation(&self) -> usize {
        self.stack
            .iter()
            .filter(|e| **e == StackEntry::Unmixed)
            .count()
    }

    fn get_newline(&self) -> bool {
        !self.in_mixed()
    }

    fn has_text_child(&self, node: Node) -> bool {
        self.xot
            .children(node)
            .any(|child| self.xot.value_type(child) == ValueType::Text)
    }

    fn indentation_newline(&mut self, node: Node) -> (usize, bool) {
        let newline = if self.xot.first_child(node).is_some() {
            if !self.has_text_child(node) {
                self.unmixed();
                self.get_newline()
            } else {
                self.mixed();
                false
            }
        } else {
            false
        };
        (0, newline)
    }

    fn serialize(&mut self, node: Node, to_be_serialized: &ToBeSerialized) -> (usize, bool) {
        use ToBeSerialized::*;
        match to_be_serialized {
            StartTagOpen(_) => (self.get_indentation(), false),
            StartTagClose(..) | Comment(..) | ProcessingInstruction(..) => {
                self.indentation_newline(node)
            }
            EndTag(_) => {
                let indentation = if self.xot.first_child(node).is_some() {
                    let was_in_mixed = self.in_mixed();
                    self.pop();
                    if !was_in_mixed {
                        self.get_indentation()
                    } else {
                        0
                    }
                } else {
                    0
                };
                (indentation, self.get_newline())
            }
            _ => (0, false),
        }
    }
}

fn serialize<'a, W: io::Write>(
    xot: &'a Xot<'a>,
    w: &mut W,
    to_be_serializeds: impl Iterator<Item = (Node, ToBeSerialized<'a>)>,
) -> Result<(), Error> {
    let xml_serializer = XmlSerializer::new(xot);
    let mut pretty_serializer = PrettySerializer::new(xot);
    for (node, to_be_serialized) in to_be_serializeds {
        let (indentation, newline) = pretty_serializer.serialize(node, &to_be_serialized);
        if indentation > 0 {
            w.write_all(" ".repeat(indentation * 2).as_bytes())?;
        }
        serialize_node(&xml_serializer, w, node, to_be_serialized)?;
        if newline {
            w.write_all(b"\n")?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pretty() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<a><b/></a>"#).unwrap();
        let extra_prefixes = ToNamespace::new();
        let iter = ToBeSerializedIterator::new(&xot, root, &extra_prefixes);
        let mut buf = Vec::new();
        serialize(&xot, &mut buf, iter).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert_eq!(s, "<a>\n  <b/>\n</a>\n")
    }
}
