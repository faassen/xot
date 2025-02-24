use crate::xmlvalue::{Namespace, Value, ValueCategory};
use crate::{NamespaceId, Node, PrefixId, ReadNode, Xot};

use super::core::{category_predicate, MutableNodeMap, NodeMap, ValueAdapter};

pub struct NamespaceAdapter {}

impl ValueAdapter<PrefixId, NamespaceId> for NamespaceAdapter {
    fn matches(value: &Value) -> bool {
        matches!(value, Value::Namespace(_))
    }

    fn children<'a, N: ReadNode + 'a>(xot: &'a Xot, node: N) -> impl Iterator<Item = N> + 'a {
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

    fn key(value: &Value) -> PrefixId {
        match value {
            Value::Namespace(Namespace { prefix_id, .. }) => *prefix_id,
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

/// A map of namespace prefixes to namespace ids.
///
/// Behaves like a HashMap, but stores the data in the tree, so that namespace
/// nodes have a parent and can exist unattached.
///
/// Access is linear time. Insertion order is preserved.
///
/// Obtained using [`Xot::namespaces`].
///
/// See [`NodeMap`] for details.
pub type Namespaces<'a, N: ReadNode> = NodeMap<'a, PrefixId, NamespaceId, NamespaceAdapter, N>;

/// A mutable map of namespace prefixes to namespace ids.
///
/// Obtained using [`Xot::namespaces_mut`].
///
/// See [`MutableNodeMap`] for details.
///
/// See also [`Namespaces`].
pub type MutableNamespaces<'a> = MutableNodeMap<'a, PrefixId, NamespaceId, NamespaceAdapter>;
