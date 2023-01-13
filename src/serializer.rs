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
pub enum Output<'a> {
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

#[generator(yield((Node, Output<'a>)))]
pub(crate) fn gen_outputs<'a>(xot: &'a Xot<'a>, node: Node) {
    let extra_prefixes = get_extra_prefixes(xot, node);
    for edge in xot.traverse(node) {
        match edge {
            NodeEdge::Start(current_node) => {
                mk_gen!(let gen = gen_edge_start(xot, node, current_node, &extra_prefixes));
                for output in gen {
                    yield_!((current_node, output));
                }
            }
            NodeEdge::End(current_node) => {
                mk_gen!(let gen = gen_edge_end(xot, current_node));
                for output in gen {
                    yield_!((current_node, output));
                }
            }
        }
    }
}

#[generator(yield(Output<'a>))]
fn gen_edge_start<'a>(xot: &'a Xot<'a>, top_node: Node, node: Node, extra_prefixes: &Prefixes) {
    let value = xot.value(node);
    match value {
        Value::Root => {}
        Value::Element(element) => {
            yield_!(Output::StartTagOpen(element));

            // serialize any extra prefixes if this is the top element of
            // a fragment and they aren't declared already
            if node == top_node {
                for (prefix_id, namespace_id) in extra_prefixes {
                    if !element.prefixes.contains_key(prefix_id) {
                        yield_!(Output::NamespaceDeclaration(
                            element,
                            *prefix_id,
                            *namespace_id,
                        ));
                    }
                }
            }

            for (prefix_id, namespace_id) in element.prefixes() {
                yield_!(Output::NamespaceDeclaration(
                    element,
                    *prefix_id,
                    *namespace_id,
                ));
            }
            yield_!(Output::NamespacesFinished(element));

            for (name_id, value) in element.attributes() {
                yield_!(Output::Attribute(element, *name_id, value));
            }
            yield_!(Output::AttributesFinished(element));

            yield_!(Output::StartTagClose(element));
        }
        Value::Text(text) => {
            yield_!(Output::Text(text.get()));
        }
        Value::Comment(comment) => {
            yield_!(Output::Comment(comment.get()));
        }
        Value::ProcessingInstruction(pi) => {
            yield_!(Output::ProcessingInstruction(pi.target(), pi.data()));
        }
    }
}

#[generator(yield(Output<'a>))]
fn gen_edge_end<'a>(xot: &'a Xot<'a>, node: Node) {
    let value = xot.value(node);
    if let Value::Element(element) = value {
        yield_!(Output::EndTag(element));
    }
}

pub(crate) struct XmlSerializer<'a> {
    xot: &'a Xot<'a>,
    fullname_serializer: FullnameSerializer<'a>,
}

pub struct Token {
    pub space: bool,
    pub text: String,
}

impl<'a> XmlSerializer<'a> {
    pub(crate) fn new(xot: &'a Xot<'a>, node: Node) -> Self {
        let extra_prefixes = get_extra_prefixes(xot, node);
        let mut fullname_serializer = FullnameSerializer::new(xot);
        fullname_serializer.push(&extra_prefixes);
        Self {
            xot,
            fullname_serializer,
        }
    }

    pub(crate) fn serialize<W: io::Write>(
        &mut self,
        w: &mut W,
        outputs: impl Iterator<Item = (Node, Output<'a>)>,
    ) -> Result<(), Error> {
        for (node, output) in outputs {
            self.serialize_node(w, node, output)?;
        }
        Ok(())
    }

    pub(crate) fn serialize_pretty<W: io::Write>(
        &mut self,
        w: &mut W,
        outputs: impl Iterator<Item = (Node, Output<'a>)>,
    ) -> Result<(), Error> {
        let mut pretty = Pretty::new(self.xot);
        for (node, output) in outputs {
            let (indentation, newline) = pretty.prettify(node, &output);
            if indentation > 0 {
                w.write_all(" ".repeat(indentation * 2).as_bytes())?;
            }
            self.serialize_node(w, node, output)?;
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
        output: Output<'a>,
    ) -> Result<(), Error> {
        let data = self.render_output(node, &output)?;
        if data.space {
            w.write_all(b" ").unwrap();
        }
        w.write_all(data.text.as_bytes()).unwrap();
        Ok(())
    }

    pub(crate) fn render_output(
        &mut self,
        node: Node,
        output: &Output<'a>,
    ) -> Result<Token, Error> {
        use Output::*;
        let r = match output {
            StartTagOpen(element) => {
                self.fullname_serializer.push(&element.prefixes);
                Token {
                    space: false,
                    text: format!(
                        "<{}",
                        self.fullname_serializer.fullname_or_err(element.name_id)?
                    ),
                }
            }
            StartTagClose(_element) => {
                if self.xot.first_child(node).is_none() {
                    Token {
                        space: false,
                        text: "/>".to_string(),
                    }
                } else {
                    Token {
                        space: false,
                        text: ">".to_string(),
                    }
                }
            }
            EndTag(element) => {
                let r = if self.xot.first_child(node).is_some() {
                    Token {
                        space: false,
                        text: format!(
                            "</{}>",
                            self.fullname_serializer.fullname_or_err(element.name_id)?
                        ),
                    }
                } else {
                    Token {
                        space: false,
                        text: "".to_string(),
                    }
                };
                self.fullname_serializer.pop(&element.prefixes);
                r
            }
            NamespaceDeclaration(_element, prefix_id, namespace_id) => {
                let namespace = self.xot.namespace_str(*namespace_id);
                if *prefix_id == self.xot.empty_prefix_id {
                    Token {
                        space: true,
                        text: format!("xmlns=\"{}\"", namespace),
                    }
                } else {
                    let prefix = self.xot.prefix_str(*prefix_id);
                    Token {
                        space: true,
                        text: format!("xmlns:{}=\"{}\"", prefix, namespace),
                    }
                }
            }
            NamespacesFinished(_element) => Token {
                space: false,
                text: "".to_string(),
            },
            Attribute(_element, name_id, value) => {
                let fullname = self.fullname_serializer.fullname_attr_or_err(*name_id)?;
                Token {
                    space: true,
                    text: format!("{}=\"{}\"", fullname, serialize_attribute((*value).into())),
                }
            }
            AttributesFinished(_element) => Token {
                space: false,
                text: "".to_string(),
            },
            Text(text) => Token {
                space: false,
                text: serialize_text((*text).into()).to_string(),
            },
            Comment(text) => Token {
                space: false,
                text: format!("<!--{}-->", text),
            },
            ProcessingInstruction(target, data) => {
                if let Some(data) = data {
                    Token {
                        space: false,
                        text: format!("<?{} {}?>", target, data),
                    }
                } else {
                    Token {
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
        mk_gen!(let mut iter = gen_outputs(&xot, doc));

        let v = iter.next().unwrap().1;
        assert_eq!(v, Output::StartTagOpen(doc_el));
        let v = iter.next().unwrap().1;
        assert_eq!(v, Output::NamespacesFinished(doc_el));
        let v = iter.next().unwrap().1;
        assert_eq!(v, Output::Attribute(doc_el, a_id, "A"));
        let v = iter.next().unwrap().1;
        assert_eq!(v, Output::AttributesFinished(doc_el));
        let v = iter.next().unwrap().1;
        assert_eq!(v, Output::StartTagClose(doc_el));
        let v = iter.next().unwrap().1;
        assert_eq!(v, Output::Text("Text"));
        let v = iter.next().unwrap().1;
        assert_eq!(v, Output::EndTag(doc_el));
        assert!(iter.next().is_none());
    }
}
