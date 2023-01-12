use indextree::{NodeEdge, Traverse};
use std::io;
use vector_map::Iter;

use crate::entity::{serialize_attribute, serialize_text};
use crate::error::Error;
use crate::fullname::FullnameSerializer;
use crate::name::NameId;
use crate::namespace::NamespaceId;
use crate::prefix::PrefixId;
use crate::xmlvalue::{Element, ToNamespace, Value};
use crate::xotdata::{Node, Xot};

#[derive(Debug, PartialEq)]
pub enum ToBeSerialized<'a> {
    StartTagOpen(&'a Element),
    StartTagClose(&'a Element),
    EndTag(&'a Element),
    NamespaceDeclaration(&'a Element, PrefixId, NamespaceId),
    NamespacesFinished(&'a Element),
    Attribute(&'a Element, NameId, &'a str),
    AttributesFinished(&'a Element),
    Text(&'a str),
    Comment(&'a str),
    ProcessingInstruction(&'a str, Option<&'a str>),
}

pub(crate) struct ToBeSerializedIterator<'a> {
    xot: &'a Xot<'a>,
    node: Node,
    traverse: Traverse<'a, Value>,
    node_edge: Option<NodeEdge>,
    phase: Phase,
    extra_prefixes: &'a ToNamespace,
    prefixes: Option<Box<dyn Iterator<Item = (PrefixId, NamespaceId)> + 'a>>,
    attributes: Option<Iter<'a, NameId, String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    StartTagOpen,
    NamespaceDeclaration,
    NamespacesFinished,
    Attribute,
    AttributesFinished,
    StartTagClose,
    Children,
}

impl<'a> ToBeSerializedIterator<'a> {
    pub(crate) fn new(xot: &'a Xot<'a>, node: Node, extra_prefixes: &'a ToNamespace) -> Self {
        let mut traverse = node.get().traverse(xot.arena());
        let node_edge = traverse.next();
        // let extra_prefixes = if let Some(parent) = xot.parent(node) {
        //     if xot.value_type(parent) != ValueType::Root {
        //         xot.to_namespace_in_scope(parent)
        //     } else {
        //         ToNamespace::new()
        //     }
        // } else {
        //     ToNamespace::new()
        // };

        Self {
            xot,
            node,
            traverse,
            node_edge,
            phase: Phase::StartTagOpen,
            extra_prefixes,
            prefixes: None,
            attributes: None,
        }
    }

    fn get_prefixes_iter(
        &self,
        element: &'a Element,
        is_top_node: bool,
    ) -> Box<dyn Iterator<Item = (PrefixId, NamespaceId)> + 'a> {
        if is_top_node {
            Box::new(
                self.extra_prefixes
                    .iter()
                    .chain(element.prefixes().iter())
                    .map(|(prefix, namespace)| (*prefix, *namespace)),
            )
        } else {
            Box::new(
                element
                    .prefixes()
                    .iter()
                    .map(|(prefix, namespace)| (*prefix, *namespace)),
            )
        }
    }

    fn element_start_next(
        &mut self,
        element: &'a Element,
        phase: Phase,
        is_top_node: bool,
    ) -> ToBeSerialized<'a> {
        match phase {
            Phase::StartTagOpen => {
                self.phase = Phase::NamespaceDeclaration;
                ToBeSerialized::StartTagOpen(element)
            }
            Phase::NamespaceDeclaration => {
                if self.prefixes.is_none() {
                    let prefixes = self.get_prefixes_iter(element, is_top_node);
                    self.prefixes = Some(prefixes);
                }
                let prefixes = self.prefixes.as_mut().unwrap();
                let entry = prefixes.next();
                if let Some(entry) = entry {
                    let (prefix, namespace) = entry;
                    ToBeSerialized::NamespaceDeclaration(element, prefix, namespace)
                } else {
                    self.prefixes = None;
                    let next_phase = Phase::NamespacesFinished;
                    self.phase = next_phase;
                    self.element_start_next(element, next_phase, is_top_node)
                }
            }
            Phase::NamespacesFinished => {
                self.phase = Phase::Attribute;
                ToBeSerialized::NamespacesFinished(element)
            }
            Phase::Attribute => {
                if self.attributes.is_none() {
                    self.attributes = Some(element.attributes().iter());
                }
                let attributes = self.attributes.as_mut().unwrap();
                let entry = attributes.next();
                if let Some(entry) = entry {
                    let (name, value) = entry;
                    ToBeSerialized::Attribute(element, *name, value)
                } else {
                    self.attributes = None;
                    let next_phase = Phase::AttributesFinished;
                    self.phase = next_phase;
                    self.element_start_next(element, next_phase, is_top_node)
                }
            }
            Phase::AttributesFinished => {
                self.phase = Phase::StartTagClose;
                ToBeSerialized::AttributesFinished(element)
            }
            Phase::StartTagClose => {
                self.phase = Phase::Children;
                ToBeSerialized::StartTagClose(element)
            }
            Phase::Children => {
                // shouldn't be here
                unreachable!();
            }
        }
    }
}

impl<'a> Iterator for ToBeSerializedIterator<'a> {
    type Item = (Node, ToBeSerialized<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node_edge) = self.node_edge {
            match node_edge {
                NodeEdge::Start(node_id) => {
                    let node = Node::new(node_id);
                    match self.xot.value(node) {
                        Value::Element(element) => {
                            if self.phase == Phase::Children {
                                self.phase = Phase::StartTagOpen;
                                self.node_edge = self.traverse.next();
                                self.next()
                            } else {
                                Some((
                                    node,
                                    self.element_start_next(element, self.phase, node == self.node),
                                ))
                            }
                        }
                        Value::Text(text) => {
                            self.node_edge = self.traverse.next();
                            Some((node, ToBeSerialized::Text(text.get())))
                        }
                        Value::Comment(comment) => {
                            self.node_edge = self.traverse.next();
                            Some((node, ToBeSerialized::Comment(comment.get())))
                        }
                        Value::ProcessingInstruction(pi) => {
                            self.node_edge = self.traverse.next();
                            Some((
                                node,
                                ToBeSerialized::ProcessingInstruction(pi.target(), pi.data()),
                            ))
                        }
                        Value::Root => {
                            self.node_edge = self.traverse.next();
                            self.next()
                        }
                    }
                }
                NodeEdge::End(node_id) => {
                    let node = Node::new(node_id);
                    if let Value::Element(element) = self.xot.value(node) {
                        self.node_edge = self.traverse.next();
                        Some((node, ToBeSerialized::EndTag(element)))
                    } else {
                        self.node_edge = self.traverse.next();
                        self.next()
                    }
                }
            }
        } else {
            None
        }
    }
}
pub(crate) struct XmlSerializer<'a> {
    xot: &'a Xot<'a>,
    fullname_serializer: FullnameSerializer<'a>,
}

pub struct SerializationData {
    space: bool,
    text: String,
}

impl<'a> XmlSerializer<'a> {
    pub(crate) fn new(xot: &'a Xot<'a>, extra_prefixes: &ToNamespace) -> Self {
        let mut fullname_serializer = FullnameSerializer::new(xot);
        fullname_serializer.push(extra_prefixes);
        Self {
            xot,
            fullname_serializer,
        }
    }

    pub(crate) fn serialize(
        &mut self,
        node: Node,
        to_be_serialized: &ToBeSerialized,
    ) -> Result<SerializationData, Error> {
        use ToBeSerialized::*;
        let r = match to_be_serialized {
            StartTagOpen(element) => {
                self.fullname_serializer
                    .push(&element.namespace_info.to_namespace);
                SerializationData {
                    space: false,
                    text: format!(
                        "<{}",
                        self.fullname_serializer.fullname_or_err(element.name_id)?
                    ),
                }
            }
            StartTagClose(_element) => {
                if self.xot.first_child(node).is_none() {
                    SerializationData {
                        space: false,
                        text: "/>".to_string(),
                    }
                } else {
                    SerializationData {
                        space: false,
                        text: ">".to_string(),
                    }
                }
            }
            EndTag(element) => {
                let r = if self.xot.first_child(node).is_some() {
                    SerializationData {
                        space: false,
                        text: format!(
                            "</{}>",
                            self.fullname_serializer.fullname_or_err(element.name_id)?
                        ),
                    }
                } else {
                    SerializationData {
                        space: false,
                        text: "".to_string(),
                    }
                };
                self.fullname_serializer
                    .pop(&element.namespace_info.to_namespace);
                r
            }
            NamespaceDeclaration(_element, prefix_id, namespace_id) => {
                let namespace = self.xot.namespace_str(*namespace_id);
                if *prefix_id == self.xot.empty_prefix_id {
                    SerializationData {
                        space: true,
                        text: format!("xmlns=\"{}\"", namespace),
                    }
                } else {
                    let prefix = self.xot.prefix_str(*prefix_id);
                    SerializationData {
                        space: true,
                        text: format!("xmlns:{}=\"{}\"", prefix, namespace),
                    }
                }
            }
            NamespacesFinished(_element) => SerializationData {
                space: false,
                text: "".to_string(),
            },
            Attribute(_element, name_id, value) => {
                let fullname = self.fullname_serializer.fullname_attr_or_err(*name_id)?;
                SerializationData {
                    space: true,
                    text: format!("{}=\"{}\"", fullname, serialize_attribute((*value).into())),
                }
            }
            AttributesFinished(_element) => SerializationData {
                space: false,
                text: "".to_string(),
            },
            Text(text) => SerializationData {
                space: false,
                text: serialize_text((*text).into()).to_string(),
            },
            Comment(text) => SerializationData {
                space: false,
                text: format!("<!--{}-->", text),
            },
            ProcessingInstruction(target, data) => {
                if let Some(data) = data {
                    SerializationData {
                        space: false,
                        text: format!("<?{} {}?>", target, data),
                    }
                } else {
                    SerializationData {
                        space: false,
                        text: format!("<?{}?>", target),
                    }
                }
            }
        };
        Ok(r)
    }
}

pub(crate) fn serialize<'a, W: io::Write>(
    xot: &'a Xot<'a>,
    w: &mut W,
    to_be_serializeds: impl Iterator<Item = (Node, ToBeSerialized<'a>)>,
    extra_prefixes: &ToNamespace,
) -> Result<(), Error> {
    let mut serializer = XmlSerializer::new(xot, extra_prefixes);
    for (node, to_be_serialized) in to_be_serializeds {
        serialize_node(&mut serializer, w, node, to_be_serialized)?;
    }
    Ok(())
}

pub(crate) fn serialize_node<W: io::Write>(
    serializer: &mut XmlSerializer,
    w: &mut W,
    node: Node,
    to_be_serialized: ToBeSerialized,
) -> Result<(), Error> {
    let data = serializer.serialize(node, &to_be_serialized)?;
    if data.space {
        w.write_all(b" ").unwrap();
    }
    w.write_all(data.text.as_bytes()).unwrap();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<doc a="A">Text</doc>"#).unwrap();
        let a_id = xot.add_name("a");
        let doc = xot.document_element(root).unwrap();
        let doc_el = xot.element(doc).unwrap();
        let extra_prefixes = ToNamespace::new();
        let mut iter = ToBeSerializedIterator::new(&xot, root, &extra_prefixes);

        let v = iter.next().unwrap().1;
        assert_eq!(v, ToBeSerialized::StartTagOpen(doc_el));
        let v = iter.next().unwrap().1;
        assert_eq!(v, ToBeSerialized::NamespacesFinished(doc_el));
        let v = iter.next().unwrap().1;
        assert_eq!(v, ToBeSerialized::Attribute(doc_el, a_id, "A"));
        let v = iter.next().unwrap().1;
        assert_eq!(v, ToBeSerialized::AttributesFinished(doc_el));
        let v = iter.next().unwrap().1;
        assert_eq!(v, ToBeSerialized::StartTagClose(doc_el));
        let v = iter.next().unwrap().1;
        assert_eq!(v, ToBeSerialized::Text("Text"));
        let v = iter.next().unwrap().1;
        assert_eq!(v, ToBeSerialized::EndTag(doc_el));
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_serialize() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<doc a="A">Text</doc>"#).unwrap();
        let extra_prefixes = ToNamespace::new();
        let iter = ToBeSerializedIterator::new(&xot, root, &extra_prefixes);
        let mut buf = Vec::new();
        let extra_prefixes = ToNamespace::new();
        serialize(&xot, &mut buf, iter, &extra_prefixes).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert_eq!(s, r#"<doc a="A">Text</doc>"#);
    }
}
