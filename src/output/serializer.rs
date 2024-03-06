use genawaiter::rc::gen;
use genawaiter::yield_;

use crate::access::NodeEdge;
use crate::id::{NameId, NamespaceId, PrefixId};
use crate::xmlvalue::{Element, Prefixes};
use crate::xmlvalue::{Value, ValueType};
use crate::xotdata::{Node, Xot};

/// Output of serialization
///
/// Given an [`OutputToken`] or
/// [`PrettyOutputToken`](`crate::output::PrettyOutputToken`), this enum
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
        for edge in xot.traverse(node) {
            match edge {
                NodeEdge::Start(current_node) => {
                    let gen = gen_edge_start(xot, node, current_node);
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

fn gen_edge_start(xot: &Xot, top_node: Node, node: Node) -> impl Iterator<Item = Output> + '_ {
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
                    for (prefix_id, namespace_id) in xot.namespaces_in_scope(node) {
                        if !namespaces.contains_key(prefix_id) {
                            yield_!(Output::Prefix(prefix_id, namespace_id,));
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
        assert_eq!(v, Output::Prefix(xot.xml_prefix(), xot.xml_namespace()));
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
