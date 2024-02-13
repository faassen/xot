// implement the HashMap API but based on nodes in the indextree

use std::borrow::Borrow;

use crate::{
    xmlvalue::{Attribute, Namespace, ValueCategory},
    NameId, NamespaceId, Node, PrefixId, Value, Xot,
};

pub trait ValueAdapter<K, V> {
    fn children(xot: &Xot, parent: Node) -> Box<dyn Iterator<Item = Node> + '_>;
    // new node insertion point is either node whether it should be inserted after,
    // or if None, prepend in the beginning
    fn insertion_point(xot: &Xot, parent: Node) -> Option<Node>;
    fn key(value: &Value) -> &K;
    fn value(value: &Value) -> &V;
    fn value_mut(value: &mut Value) -> &mut V;
    fn create(key: K, value: V) -> Value;
    fn update(value: &mut Value, value: V) -> Option<V>;
}

#[derive(Debug)]
pub struct NodeMap<'a, K, V, A: ValueAdapter<K, V>>
where
    K: PartialEq,
{
    xot: &'a mut Xot,
    parent: Node,
    _a: std::marker::PhantomData<A>,
    _k: std::marker::PhantomData<K>,
    _v: std::marker::PhantomData<V>,
}

impl<'a, K, V, A: ValueAdapter<K, V>> NodeMap<'a, K, V, A>
where
    K: PartialEq,
{
    fn new(xot: &'a mut Xot, parent: Node) -> Self {
        NodeMap {
            xot,
            parent,
            _a: std::marker::PhantomData,
            _k: std::marker::PhantomData,
            _v: std::marker::PhantomData,
        }
    }

    fn children(&self) -> Box<dyn Iterator<Item = Node> + '_> {
        A::children(self.xot, self.parent)
    }

    /// Returns the number of entries in the map, also referred to as its 'length'.
    pub fn len(&self) -> usize {
        self.children().count()
    }

    /// Returns `true` if the map contains no entries.
    pub fn is_empty(&self) -> bool {
        self.children().next().is_some()
    }

    /// Clears the map, removing all entries.
    pub fn clear(&mut self) {
        let to_remove = self.children().collect::<Vec<_>>();

        for child in to_remove {
            self.xot.remove(child);
        }
    }

    // TODO: retain, drain, sort_keys, sort_unstable_keys, sort_by, sort_unstable_by,

    /// Return `true` if an equivalent to `key` exists in the map.
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        for child in self.children() {
            if A::key(self.xot.value(child)).borrow() == key {
                return true;
            }
        }
        false
    }

    fn get_node_q<Q>(&self, key: &Q) -> Option<Node>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.children()
            .find(|&child| A::key(self.xot.value(child)).borrow() == key)
    }

    fn get_node(&self, key: &K) -> Option<Node> {
        self.children()
            .find(|&child| A::key(self.xot.value(child)) == key)
    }

    /// Return a reference to the value stored for `key`, if it is present, else `None`.
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        let node = self.get_node_q(key)?;
        Some(A::value(self.xot.value(node)))
    }

    /// Return a mutable reference to the value stored for `key`, if it is present, else `None`.
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        let node = self.get_node_q(key)?;
        Some(A::value_mut(self.xot.value_mut(node)))
    }

    // todo: get_key_value

    // todo: pop, remove, remove_entry

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let node = self.get_node(&key);
        if let Some(node) = node {
            // if we already have a node
            let node_value = self.xot.value_mut(node);
            A::update(node_value, value)
        } else {
            // we need to insert a new node
            let new_value = A::create(key, value);
            let node = self.xot.arena.new_node(new_value);
            let insertion_point = A::insertion_point(self.xot, self.parent);
            if let Some(insertion_point) = insertion_point {
                insertion_point
                    .get()
                    .checked_insert_after(node, &mut self.xot.arena)
                    .unwrap();
            } else {
                self.parent
                    .get()
                    .checked_prepend(node, &mut self.xot.arena)
                    .unwrap();
            }
            None
        }
    }
}

struct AttributeAdapter {}

fn category_predicate(xot: &Xot, category: ValueCategory) -> impl Fn(&Node) -> bool + '_ {
    move |node| xot.value(*node).value_category() == category
}

impl ValueAdapter<NameId, String> for AttributeAdapter {
    fn children(xot: &Xot, node: Node) -> Box<dyn Iterator<Item = Node> + '_> {
        Box::new(
            xot.all_children(node)
                .skip_while(category_predicate(xot, ValueCategory::Namespace))
                .take_while(category_predicate(xot, ValueCategory::Attribute)),
        )
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

struct NamespaceAdapter {}

impl ValueAdapter<PrefixId, NamespaceId> for NamespaceAdapter {
    fn children(xot: &Xot, node: Node) -> Box<dyn Iterator<Item = Node> + '_> {
        Box::new(
            xot.all_children(node)
                .take_while(category_predicate(xot, ValueCategory::Namespace)),
        )
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

pub type Attributes<'a> = NodeMap<'a, NameId, String, AttributeAdapter>;
pub type Namespaces<'a> = NodeMap<'a, PrefixId, NamespaceId, NamespaceAdapter>;

#[cfg(test)]
mod tests {
    use super::*;

    use crate::Xot;

    #[test]
    fn test_attribute_get() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<doc a="A"></doc>"#);
        let a = xot.add_name("a");
        let document_element = xot.document_element(root.unwrap()).unwrap();
        let attributes = Attributes::new(&mut xot, document_element);
        assert_eq!(attributes.get(&a), Some(&"A".to_string()));
    }

    #[test]
    fn test_attribute_insert() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<doc a="A"></doc>"#);
        let a = xot.add_name("a");
        let document_element = xot.document_element(root.unwrap()).unwrap();
        let mut attributes = Attributes::new(&mut xot, document_element);
        attributes.insert(a, "B".to_string());
        assert_eq!(attributes.get(&a), Some(&"B".to_string()));
    }

    #[test]
    fn test_attribute_insert_new_blank() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<doc></doc>"#);
        let a = xot.add_name("a");
        let document_element = xot.document_element(root.unwrap()).unwrap();
        let mut attributes = Attributes::new(&mut xot, document_element);
        attributes.insert(a, "A".to_string());
        assert_eq!(attributes.get(&a), Some(&"A".to_string()));
    }

    #[test]
    fn test_attribute_insert_new_existing_attributes() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<doc c="C"></doc>"#);
        let a = xot.add_name("a");
        let document_element = xot.document_element(root.unwrap()).unwrap();
        let mut attributes = Attributes::new(&mut xot, document_element);
        attributes.insert(a, "A".to_string());
        assert_eq!(attributes.get(&a), Some(&"A".to_string()));
    }

    #[test]
    fn test_attributes_and_namespaces() {
        let mut xot = Xot::new();

        let root = xot.parse(r#"<doc xmlns:foo="FOO" a="A"></doc>"#);
        let a = xot.add_name("a");
        let foo_prefix = xot.add_prefix("foo");
        let foo_ns = xot.add_namespace("FOO");
        let document_element = xot.document_element(root.unwrap()).unwrap();
        let attributes = Attributes::new(&mut xot, document_element);
        assert_eq!(attributes.get(&a), Some(&"A".to_string()));
        let namespaces = Namespaces::new(&mut xot, document_element);
        assert_eq!(namespaces.get(&foo_prefix), Some(&foo_ns));
    }
}
