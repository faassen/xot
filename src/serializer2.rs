use std::io;

use crate::entity::{serialize_attribute, serialize_text};
use crate::error::Error;
use crate::fullname::FullnameSerializer;
use crate::name::NameId;
use crate::namespace::NamespaceId;
use crate::prefix::PrefixId;
use crate::xmlvalue::{Element, ToNamespace};
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
