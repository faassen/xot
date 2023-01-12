use next_gen::prelude::*;

use crate::access::NodeEdge;
use crate::serializer2::ToBeSerialized;
use crate::xmlvalue::{ToNamespace, Value, ValueType};
use crate::xotdata::{Node, Xot};

pub(crate) fn get_extra_prefixes(xot: &Xot, node: Node) -> ToNamespace {
    // collect namespace prefixes for all ancestors of the fragment
    if let Some(parent) = xot.parent(node) {
        if xot.value_type(parent) != ValueType::Root {
            xot.to_namespace_in_scope(parent)
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
#[generator(yield((Node, ToBeSerialized<'a>)))]
pub(crate) fn serialize_node<'a>(xot: &'a Xot<'a>, node: Node, extra_prefixes: &ToNamespace) {
    for edge in xot.traverse(node) {
        match edge {
            NodeEdge::Start(current_node) => {
                mk_gen!(let gen = handle_edge_start(xot, node, current_node, extra_prefixes));
                for to_be_serialized in gen {
                    yield_!((current_node, to_be_serialized));
                }
            }
            NodeEdge::End(current_node) => {
                mk_gen!(let gen = handle_edge_end(xot, current_node));
                for to_be_serialized in gen {
                    yield_!((current_node, to_be_serialized));
                }
            }
        }
    }
}

#[generator(yield(ToBeSerialized<'a>))]
fn handle_edge_start<'a>(
    xot: &'a Xot<'a>,
    top_node: Node,
    node: Node,
    extra_prefixes: &ToNamespace,
) {
    let value = xot.value(node);
    match value {
        Value::Root => {}
        Value::Element(element) => {
            yield_!(ToBeSerialized::StartTagOpen(element));

            // serialize any extra prefixes if this is the top element of
            // a fragment and they aren't declared already
            if node == top_node {
                for (prefix_id, namespace_id) in extra_prefixes {
                    if !element.namespace_info.to_namespace.contains_key(prefix_id) {
                        yield_!(ToBeSerialized::NamespaceDeclaration(
                            element,
                            *prefix_id,
                            *namespace_id,
                        ));
                    }
                }
            }

            for (prefix_id, namespace_id) in element.prefixes() {
                yield_!(ToBeSerialized::NamespaceDeclaration(
                    element,
                    *prefix_id,
                    *namespace_id,
                ));
            }
            yield_!(ToBeSerialized::NamespacesFinished(element));

            for (name_id, value) in element.attributes() {
                yield_!(ToBeSerialized::Attribute(element, *name_id, value));
            }
            yield_!(ToBeSerialized::AttributesFinished(element));

            yield_!(ToBeSerialized::StartTagClose(element));
        }
        Value::Text(text) => {
            yield_!(ToBeSerialized::Text(text.get()));
        }
        Value::Comment(comment) => {
            yield_!(ToBeSerialized::Comment(comment.get()));
        }
        Value::ProcessingInstruction(pi) => {
            yield_!(ToBeSerialized::ProcessingInstruction(
                pi.target(),
                pi.data()
            ));
        }
    }
}
#[generator(yield(ToBeSerialized<'a>))]
fn handle_edge_end<'a>(xot: &'a Xot<'a>, node: Node) {
    let value = xot.value(node);
    if let Value::Element(element) = value {
        yield_!(ToBeSerialized::EndTag(element));
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
        let extra_prefixes = ToNamespace::new();
        mk_gen!(let mut iter = serialize_node(&xot, doc, &extra_prefixes));

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
}
