use crate::xmlvalue::{Namespace, Value, ValueCategory};
use crate::{NamespaceId, Node, PrefixId, Xot};

use super::core::{category_predicate, NodeMap, ValueAdapter};

pub struct NamespaceAdapter {}

impl ValueAdapter<PrefixId, NamespaceId> for NamespaceAdapter {
    fn children(xot: &Xot, node: Node) -> impl Iterator<Item = Node> + '_ {
        xot.all_children(node)
            .take_while(category_predicate(xot, ValueCategory::Namespace))
    }

    fn insertion_point(xot: &Xot, node: Node) -> Option<Node> {
        let last_child = Self::children(xot, node).last();
        // if there is a last child, insert after it
        if let Some(last_child) = last_child {
            return Some(last_child);
        }
        // if there is no namespace node, we want to prepend
        None
    }

    fn key(value: &Value) -> &PrefixId {
        match value {
            Value::Namespace(Namespace { prefix_id, .. }) => prefix_id,
            _ => unreachable!(),
        }
    }

    fn value(value: &Value) -> &NamespaceId {
        match value {
            Value::Namespace(Namespace { namespace_id, .. }) => namespace_id,
            _ => unreachable!(),
        }
    }

    fn value_mut(value: &mut Value) -> &mut NamespaceId {
        match value {
            Value::Namespace(Namespace { namespace_id, .. }) => namespace_id,
            _ => unreachable!(),
        }
    }

    fn create(key: PrefixId, value: NamespaceId) -> Value {
        Value::Namespace(Namespace {
            prefix_id: key,
            namespace_id: value,
        })
    }

    fn update(value: &mut Value, new_value: NamespaceId) -> Option<NamespaceId> {
        match value {
            Value::Namespace(Namespace {
                namespace_id: old_value,
                ..
            }) => {
                let old_value = std::mem::replace(old_value, new_value);
                Some(old_value)
            }
            _ => unreachable!(),
        }
    }
}

pub type Namespaces<'a> = NodeMap<'a, PrefixId, NamespaceId, NamespaceAdapter>;
