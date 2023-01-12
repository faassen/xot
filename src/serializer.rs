use next_gen::prelude::*;
use std::io;

use crate::access::NodeEdge;
use crate::entity::{serialize_attribute, serialize_text};
use crate::error::Error;
use crate::fullname::FullnameSerializer;
use crate::name::NameId;
use crate::namespace::NamespaceId;
use crate::prefix::PrefixId;
use crate::pretty::Pretty;
use crate::xmlvalue::{Element, Prefixes};
use crate::xmlvalue::{Value, ValueType};
use crate::xotdata::{Node, Xot};

#[derive(Debug, PartialEq)]
pub enum OutputToken<'a> {
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

#[generator(yield((Node, OutputToken<'a>)))]
pub(crate) fn gen_tokens<'a>(xot: &'a Xot<'a>, node: Node, extra_prefixes: &Prefixes) {
    for edge in xot.traverse(node) {
        match edge {
            NodeEdge::Start(current_node) => {
                mk_gen!(let gen = gen_edge_start(xot, node, current_node, extra_prefixes));
                for output_token in gen {
                    yield_!((current_node, output_token));
                }
            }
            NodeEdge::End(current_node) => {
                mk_gen!(let gen = gen_edge_end(xot, current_node));
                for output_token in gen {
                    yield_!((current_node, output_token));
                }
            }
        }
    }
}

#[generator(yield(OutputToken<'a>))]
fn gen_edge_start<'a>(xot: &'a Xot<'a>, top_node: Node, node: Node, extra_prefixes: &Prefixes) {
    let value = xot.value(node);
    match value {
        Value::Root => {}
        Value::Element(element) => {
            yield_!(OutputToken::StartTagOpen(element));

            // serialize any extra prefixes if this is the top element of
            // a fragment and they aren't declared already
            if node == top_node {
                for (prefix_id, namespace_id) in extra_prefixes {
                    if !element.prefixes.contains_key(prefix_id) {
                        yield_!(OutputToken::NamespaceDeclaration(
                            element,
                            *prefix_id,
                            *namespace_id,
                        ));
                    }
                }
            }

            for (prefix_id, namespace_id) in element.prefixes() {
                yield_!(OutputToken::NamespaceDeclaration(
                    element,
                    *prefix_id,
                    *namespace_id,
                ));
            }
            yield_!(OutputToken::NamespacesFinished(element));

            for (name_id, value) in element.attributes() {
                yield_!(OutputToken::Attribute(element, *name_id, value));
            }
            yield_!(OutputToken::AttributesFinished(element));

            yield_!(OutputToken::StartTagClose(element));
        }
        Value::Text(text) => {
            yield_!(OutputToken::Text(text.get()));
        }
        Value::Comment(comment) => {
            yield_!(OutputToken::Comment(comment.get()));
        }
        Value::ProcessingInstruction(pi) => {
            yield_!(OutputToken::ProcessingInstruction(pi.target(), pi.data()));
        }
    }
}

#[generator(yield(OutputToken<'a>))]
fn gen_edge_end<'a>(xot: &'a Xot<'a>, node: Node) {
    let value = xot.value(node);
    if let Value::Element(element) = value {
        yield_!(OutputToken::EndTag(element));
    }
}

pub(crate) struct XmlSerializer<'a> {
    xot: &'a Xot<'a>,
    fullname_serializer: FullnameSerializer<'a>,
}

pub struct SerializationData {
    pub space: bool,
    pub text: String,
}

impl<'a> XmlSerializer<'a> {
    pub(crate) fn new(xot: &'a Xot<'a>, extra_prefixes: &Prefixes) -> Self {
        let mut fullname_serializer = FullnameSerializer::new(xot);
        fullname_serializer.push(extra_prefixes);
        Self {
            xot,
            fullname_serializer,
        }
    }

    pub(crate) fn serialize<W: io::Write>(
        &mut self,
        w: &mut W,
        output_tokens: impl Iterator<Item = (Node, OutputToken<'a>)>,
    ) -> Result<(), Error> {
        for (node, output_token) in output_tokens {
            self.serialize_node(w, node, output_token)?;
        }
        Ok(())
    }

    pub(crate) fn serialize_pretty<W: io::Write>(
        &mut self,
        w: &mut W,
        output_tokens: impl Iterator<Item = (Node, OutputToken<'a>)>,
    ) -> Result<(), Error> {
        let mut pretty = Pretty::new(self.xot);
        for (node, output_token) in output_tokens {
            let (indentation, newline) = pretty.prettify(node, &output_token);
            if indentation > 0 {
                w.write_all(" ".repeat(indentation * 2).as_bytes())?;
            }
            self.serialize_node(w, node, output_token)?;
            if newline {
                w.write_all(b"\n")?;
            }
        }
        Ok(())
    }

    pub(crate) fn serialize_node<W: io::Write>(
        &mut self,
        w: &mut W,
        node: Node,
        output_token: OutputToken<'a>,
    ) -> Result<(), Error> {
        let data = self.render_token(node, output_token)?;
        if data.space {
            w.write_all(b" ").unwrap();
        }
        w.write_all(data.text.as_bytes()).unwrap();
        Ok(())
    }

    pub(crate) fn render_token(
        &mut self,
        node: Node,
        output_token: OutputToken<'a>,
    ) -> Result<SerializationData, Error> {
        use OutputToken::*;
        let r = match output_token {
            StartTagOpen(element) => {
                self.fullname_serializer.push(&element.prefixes);
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
                self.fullname_serializer.pop(&element.prefixes);
                r
            }
            NamespaceDeclaration(_element, prefix_id, namespace_id) => {
                let namespace = self.xot.namespace_str(namespace_id);
                if prefix_id == self.xot.empty_prefix_id {
                    SerializationData {
                        space: true,
                        text: format!("xmlns=\"{}\"", namespace),
                    }
                } else {
                    let prefix = self.xot.prefix_str(prefix_id);
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
                let fullname = self.fullname_serializer.fullname_attr_or_err(name_id)?;
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

pub(crate) fn get_extra_prefixes(xot: &Xot, node: Node) -> Prefixes {
    // collect namespace prefixes for all ancestors of the fragment
    if let Some(parent) = xot.parent(node) {
        if xot.value_type(parent) != ValueType::Root {
            xot.prefixes_in_scope(parent)
        } else {
            Prefixes::new()
        }
    } else {
        Prefixes::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter_mkgen() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<doc a="A">Text</doc>"#).unwrap();
        let a_id = xot.add_name("a");
        let doc = xot.document_element(root).unwrap();
        let doc_el = xot.element(doc).unwrap();
        let extra_prefixes = Prefixes::new();
        mk_gen!(let mut iter = gen_tokens(&xot, doc, &extra_prefixes));

        let v = iter.next().unwrap().1;
        assert_eq!(v, OutputToken::StartTagOpen(doc_el));
        let v = iter.next().unwrap().1;
        assert_eq!(v, OutputToken::NamespacesFinished(doc_el));
        let v = iter.next().unwrap().1;
        assert_eq!(v, OutputToken::Attribute(doc_el, a_id, "A"));
        let v = iter.next().unwrap().1;
        assert_eq!(v, OutputToken::AttributesFinished(doc_el));
        let v = iter.next().unwrap().1;
        assert_eq!(v, OutputToken::StartTagClose(doc_el));
        let v = iter.next().unwrap().1;
        assert_eq!(v, OutputToken::Text("Text"));
        let v = iter.next().unwrap().1;
        assert_eq!(v, OutputToken::EndTag(doc_el));
        assert!(iter.next().is_none());
    }
}
