use genawaiter::rc::gen;
use genawaiter::yield_;
use std::io;
use std::rc::Rc;

use crate::access::NodeEdge;
use crate::entity::{serialize_attribute, serialize_cdata, serialize_text, Normalizer};
use crate::error::Error;
use crate::fullname::FullnameSerializer;
use crate::id::{NameId, NamespaceId, PrefixId};
use crate::pretty::Pretty;
use crate::xmlvalue::{Element, Prefixes};
use crate::xmlvalue::{Value, ValueType};
use crate::xotdata::{Node, Xot};

/// Output of serialization
///
/// Given an [`OutputToken`] or
/// [`PrettyOutputToken`](`crate::serialize::PrettyOutputToken`), this enum
/// represents what the token represents in the XML tree.
///
/// You can use this information for customized serialization.
#[derive(Debug, PartialEq)]
pub enum Output<'a> {
    /// Start tag open, i.e `<foo` or `<ns:foo`
    StartTagOpen(Element),
    /// Start tag close, either `>` or `/>`
    StartTagClose,
    /// End tag, i.e. `</foo>` or `</ns:foo>`
    EndTag(Element),
    /// Namespace declaration, i.e. `xmlns:foo="http://example.com"`
    Prefix(PrefixId, NamespaceId),
    /// Attribute, i.e. `foo="bar"`
    Attribute(NameId, &'a str),
    /// Text, i.e. `foo`
    Text(&'a str),
    /// Comment, i.e. `<!-- foo -->`
    Comment(&'a str),
    /// Processing instruction, i.e. `<?foo bar?>`
    ProcessingInstruction(NameId, Option<&'a str>),
}

pub(crate) fn gen_outputs(xot: &Xot, node: Node) -> impl Iterator<Item = (Node, Output)> + '_ {
    gen!({
        let extra_prefixes = Rc::new(get_extra_prefixes(xot, node));
        for edge in xot.traverse(node) {
            match edge {
                NodeEdge::Start(current_node) => {
                    let gen = gen_edge_start(xot, node, current_node, extra_prefixes.clone());
                    for output in gen {
                        yield_!((current_node, output));
                    }
                }
                NodeEdge::End(current_node) => {
                    let gen = gen_edge_end(xot, current_node);
                    for output in gen {
                        yield_!((current_node, output));
                    }
                }
            }
        }
    })
    .into_iter()
}

fn gen_edge_start(
    xot: &Xot,
    top_node: Node,
    node: Node,
    extra_prefixes: Rc<Prefixes>,
) -> impl Iterator<Item = Output> + '_ {
    gen!({
        let value = xot.value(node);

        match value {
            Value::Document => {}
            Value::Element(element) => {
                yield_!(Output::StartTagOpen(*element));

                // serialize any extra prefixes if this is the top element of
                // a fragment and they aren't declared already
                let namespaces = xot.namespaces(node);
                if node == top_node {
                    for (prefix_id, namespace_id) in extra_prefixes.iter() {
                        if !namespaces.contains_key(*prefix_id) {
                            yield_!(Output::Prefix(*prefix_id, *namespace_id,));
                        }
                    }
                }

                for (prefix_id, namespace_id) in namespaces.iter() {
                    yield_!(Output::Prefix(prefix_id, *namespace_id,));
                }

                for (name_id, value) in xot.attributes(node).iter() {
                    yield_!(Output::Attribute(name_id, value));
                }

                yield_!(Output::StartTagClose);
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
            Value::Attribute(_) | Value::Namespace(_) => {
                // handled in element
            }
        }
    })
    .into_iter()
}

fn gen_edge_end(xot: &Xot, node: Node) -> impl Iterator<Item = Output> + '_ {
    gen!({
        let value = xot.value(node);
        if let Value::Element(element) = value {
            yield_!(Output::EndTag(*element));
        }
    })
    .into_iter()
}

pub(crate) struct XmlSerializer<'a, N: Normalizer> {
    xot: &'a Xot,
    cdata_section_names: &'a [NameId],
    fullname_serializer: FullnameSerializer<'a>,
    normalizer: N,
}

/// Output token
///
/// This represents an [`Output`] as a rendered output token.
pub struct OutputToken {
    /// Whether the token is prefixed by a space character.
    pub space: bool,
    /// The token.
    ///
    /// This is a fragment of XML like `<foo` or `a="A"` or `/>`, etc.
    pub text: String,
}

impl<'a, N: Normalizer> XmlSerializer<'a, N> {
    pub(crate) fn new(
        xot: &'a Xot,
        node: Node,
        cdata_section_names: &'a [NameId],
        normalizer: N,
    ) -> Self {
        let extra_prefixes = get_extra_prefixes(xot, node);
        let mut fullname_serializer = FullnameSerializer::new(xot);
        fullname_serializer.push(&extra_prefixes);
        Self {
            xot,
            cdata_section_names,
            fullname_serializer,
            normalizer,
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
        suppress: &[NameId],
    ) -> Result<(), Error> {
        let mut pretty = Pretty::new(self.xot, suppress);
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
    ) -> Result<OutputToken, Error> {
        use Output::*;
        let r = match output {
            StartTagOpen(element) => {
                self.fullname_serializer.push(&self.xot.prefixes(node));
                OutputToken {
                    space: false,
                    text: format!(
                        "<{}",
                        self.fullname_serializer.fullname_or_err(element.name_id)?
                    ),
                }
            }
            StartTagClose => {
                if self.xot.first_child(node).is_none() {
                    OutputToken {
                        space: false,
                        text: "/>".to_string(),
                    }
                } else {
                    OutputToken {
                        space: false,
                        text: ">".to_string(),
                    }
                }
            }
            EndTag(element) => {
                let r = if self.xot.first_child(node).is_some() {
                    OutputToken {
                        space: false,
                        text: format!(
                            "</{}>",
                            self.fullname_serializer.fullname_or_err(element.name_id)?
                        ),
                    }
                } else {
                    OutputToken {
                        space: false,
                        text: "".to_string(),
                    }
                };
                self.fullname_serializer.pop();
                r
            }
            Prefix(prefix_id, namespace_id) => {
                let namespace = self.xot.namespace_str(*namespace_id);
                if *prefix_id == self.xot.empty_prefix_id {
                    OutputToken {
                        space: true,
                        text: format!("xmlns=\"{}\"", namespace),
                    }
                } else {
                    let prefix = self.xot.prefix_str(*prefix_id);
                    OutputToken {
                        space: true,
                        text: format!("xmlns:{}=\"{}\"", prefix, namespace),
                    }
                }
            }
            Attribute(name_id, value) => {
                let fullname = self.fullname_serializer.fullname_attr_or_err(*name_id)?;
                OutputToken {
                    space: true,
                    text: format!(
                        "{}=\"{}\"",
                        fullname,
                        serialize_attribute((*value).into(), &self.normalizer)
                    ),
                }
            }
            Text(text) => {
                // a text node is always a child of an element
                let parent = self.xot.parent(node).unwrap();
                let element = self.xot.element(parent).unwrap();
                if self.cdata_section_names.contains(&element.name()) {
                    OutputToken {
                        space: false,
                        text: serialize_cdata((*text).into(), &self.normalizer).to_string(),
                    }
                } else {
                    OutputToken {
                        space: false,
                        text: serialize_text((*text).into(), &self.normalizer).to_string(),
                    }
                }
            }
            Comment(text) => OutputToken {
                space: false,
                text: format!("<!--{}-->", text),
            },
            ProcessingInstruction(target, data) => {
                let (target, ns) = self.xot.name_ns_str(*target);
                if !ns.is_empty() {
                    return Err(Error::NamespaceInProcessingInstruction);
                }
                if let Some(data) = data {
                    OutputToken {
                        space: false,
                        text: format!("<?{} {}?>", target, data),
                    }
                } else {
                    OutputToken {
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
        if xot.value_type(parent) != ValueType::Document {
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
        let mut iter = gen_outputs(&xot, doc);

        let v = iter.next().unwrap().1;
        assert_eq!(v, Output::StartTagOpen(*doc_el));
        let v = iter.next().unwrap().1;
        assert_eq!(v, Output::Attribute(a_id, "A"));
        let v = iter.next().unwrap().1;
        assert_eq!(v, Output::StartTagClose);
        let v = iter.next().unwrap().1;
        assert_eq!(v, Output::Text("Text"));
        let v = iter.next().unwrap().1;
        assert_eq!(v, Output::EndTag(*doc_el));
        assert!(iter.next().is_none());
    }
}
