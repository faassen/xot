use crate::xmlvalue::{Attribute, Value, ValueCategory};
use crate::{NameId, Node, Xot};

use super::core::{category_predicate, NodeMap, ValueAdapter};

pub struct AttributeAdapter {}

impl ValueAdapter<NameId, String> for AttributeAdapter {
    fn children(xot: &Xot, node: Node) -> impl Iterator<Item = Node> + '_ {
        xot.all_children(node)
            .skip_while(category_predicate(xot, ValueCategory::Namespace))
            .take_while(category_predicate(xot, ValueCategory::Attribute))
    }

    fn insertion_point(xot: &Xot, node: Node) -> Option<Node> {
        let last_child = Self::children(xot, node).last();
        // if there is a last child, insert after it
        if let Some(last_child) = last_child {
            return Some(last_child);
        }
        // if there is no last child, then insert after the last namespace node
        let namespaces = xot
            .all_children(node)
            .take_while(category_predicate(xot, ValueCategory::Namespace));
        if let Some(last_namespace) = namespaces.last() {
            return Some(last_namespace);
        }
        // if there is no namespace node, we want to prepend
        None
    }

    fn key(value: &Value) -> &NameId {
        match value {
            Value::Attribute(Attribute { name_id, .. }) => name_id,
            _ => unreachable!(),
        }
    }

    fn value(value: &Value) -> &String {
        match value {
            Value::Attribute(Attribute { value, .. }) => value,
            _ => unreachable!(),
        }
    }

    fn value_mut(value: &mut Value) -> &mut String {
        match value {
            Value::Attribute(Attribute { value, .. }) => value,
            _ => unreachable!(),
        }
    }

    fn create(key: NameId, value: String) -> Value {
        Value::Attribute(Attribute {
            name_id: key,
            value,
        })
    }

    fn update(value: &mut Value, new_value: String) -> Option<String> {
        match value {
            Value::Attribute(Attribute {
                value: old_value, ..
            }) => {
                let old_value = std::mem::replace(old_value, new_value);
                Some(old_value)
            }
            _ => unreachable!(),
        }
    }
}

/// Attributes of an element.
///
/// Behaves like a HashMap, but stores the data in the tree, so that namespace
/// nodes have a parent and can exist unattached.
///
/// Access is linear time. Insertion order is preserved.
pub type Attributes<'a> = NodeMap<'a, NameId, String, AttributeAdapter>;
