// implement the HashMap API but based on nodes in the indextree

use std::borrow::Borrow;

use crate::{
    xmlvalue::FullValueCategory, Attribute, FullValue, NameId, Namespace, NamespaceId, PrefixId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Category {
    Namespace,
    Attribute,
}

impl From<Category> for FullValueCategory {
    fn from(category: Category) -> FullValueCategory {
        match category {
            Category::Namespace => FullValueCategory::Namespace,
            Category::Attribute => FullValueCategory::Attribute,
        }
    }
}

pub trait ValueAdapter<K, V> {
    fn key(full_value: &FullValue) -> &K;
    fn value(full_value: &FullValue) -> &V;
    fn value_mut(full_value: &mut FullValue) -> &mut V;
    fn create(key: K, value: V) -> FullValue;
    fn update(full_value: &mut FullValue, value: V) -> Option<V>;
}

#[derive(Debug)]
pub struct NodeMap<'a, K, V, A: ValueAdapter<K, V>>
where
    K: PartialEq,
{
    arena: &'a mut indextree::Arena<FullValue>,
    parent: indextree::NodeId,
    full_value_category: FullValueCategory,
    category: Category,
    _a: std::marker::PhantomData<A>,
    _k: std::marker::PhantomData<K>,
    _v: std::marker::PhantomData<V>,
}

impl<'a, K, V, A: ValueAdapter<K, V>> NodeMap<'a, K, V, A>
where
    K: PartialEq,
{
    fn new(
        arena: &'a mut indextree::Arena<FullValue>,
        parent: indextree::NodeId,
        category: Category,
    ) -> Self {
        NodeMap {
            arena,
            parent,
            full_value_category: category.into(),
            category,
            _a: std::marker::PhantomData,
            _k: std::marker::PhantomData,
            _v: std::marker::PhantomData,
        }
    }

    fn filter_by_category(
        &self,
        full_value_category: FullValueCategory,
    ) -> impl Fn(&indextree::NodeId) -> bool + '_ {
        move |node_id| self.arena[*node_id].get().full_value_category() == full_value_category
    }

    fn children(&self) -> impl Iterator<Item = indextree::NodeId> + '_ {
        self.parent
            .children(self.arena)
            .filter(self.filter_by_category(self.full_value_category))
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
            child.remove_subtree(self.arena);
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
            todo!()
        }
        todo!()
    }

    fn get_node_q<Q>(&self, key: &Q) -> Option<indextree::NodeId>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        for child in self.children() {
            let full_value = self.arena[child].get();
            if A::key(full_value).borrow() == key {
                return Some(child);
            }
        }
        None
    }

    fn get_node(&self, key: &K) -> Option<indextree::NodeId> {
        for child in self.children() {
            let full_value = self.arena[child].get();
            if A::key(full_value) == key {
                return Some(child);
            }
        }
        None
    }

    /// Return a reference to the value stored for `key`, if it is present, else `None`.
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        let node = self.get_node_q(key)?;
        let full_value = self.arena[node].get();
        Some(A::value(full_value))
    }

    /// Return a mutable reference to the value stored for `key`, if it is present, else `None`.
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        let node = self.get_node_q(key)?;
        let full_value = self.arena[node].get_mut();
        Some(A::value_mut(full_value))
    }

    // todo: get_key_value

    // todo: pop, remove, remove_entry

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let node = self.get_node(&key);
        if let Some(node) = node {
            // if we already have a node
            let full_value = self.arena[node].get_mut();
            A::update(full_value, value)
        } else {
            // we need to add the new node at the end of the category
            // or if the category is empty, create the new category
            let children = self.children();
            if let Some(last) = children.last() {
                let new_full_value = A::create(key, value);
                let node = self.arena.new_node(new_full_value);
                last.checked_insert_after(node, self.arena).unwrap();
            } else {
                // determine insertion point
                match self.category {
                    Category::Namespace => {
                        // insert at the beginning
                        let new_full_value = A::create(key, value);
                        let node = self.arena.new_node(new_full_value);
                        self.parent.checked_prepend(node, self.arena).unwrap();
                    }
                    Category::Attribute => {
                        let new_full_value = A::create(key, value);
                        let node = self.arena.new_node(new_full_value);
                        // insert after namespaces, or prepend if no namespaces
                        let namespaces = self
                            .parent
                            .children(self.arena)
                            .filter(self.filter_by_category(FullValueCategory::Namespace));
                        if let Some(last) = namespaces.last() {
                            last.checked_insert_after(node, self.arena).unwrap();
                        } else {
                            self.parent.checked_prepend(node, self.arena).unwrap();
                        }
                    }
                }
            }
            None
        }

        // we should insert at the end of our category if the key is new
        // let children = self.children();
        // if let Some(last) = children.last() {
        //     let new_full_value = (self.new_full_value)(key, value);
        //     last.checked_insert_after(new_full_value, self.arena)
        //         .unwrap();
        // }
        // if the category is empty, we should take special measure
    }
}

struct AttributeAdapter {}

impl ValueAdapter<NameId, String> for AttributeAdapter {
    fn key(full_value: &FullValue) -> &NameId {
        match full_value {
            FullValue::Attribute(Attribute { name, .. }) => name,
            _ => unreachable!(),
        }
    }

    fn value(full_value: &FullValue) -> &String {
        match full_value {
            FullValue::Attribute(Attribute { value, .. }) => value,
            _ => unreachable!(),
        }
    }

    fn value_mut(full_value: &mut FullValue) -> &mut String {
        match full_value {
            FullValue::Attribute(Attribute { value, .. }) => value,
            _ => unreachable!(),
        }
    }

    fn create(key: NameId, value: String) -> FullValue {
        FullValue::Attribute(Attribute { name: key, value })
    }

    fn update(full_value: &mut FullValue, value: String) -> Option<String> {
        match full_value {
            FullValue::Attribute(Attribute {
                value: old_value, ..
            }) => {
                let old_value = std::mem::replace(old_value, value);
                Some(old_value)
            }
            _ => unreachable!(),
        }
    }
}

struct NamespaceAdapter {}

impl ValueAdapter<PrefixId, NamespaceId> for NamespaceAdapter {
    fn key(full_value: &FullValue) -> &PrefixId {
        match full_value {
            FullValue::Namespace(Namespace { prefix, .. }) => prefix,
            _ => unreachable!(),
        }
    }

    fn value(full_value: &FullValue) -> &NamespaceId {
        match full_value {
            FullValue::Namespace(Namespace { uri, .. }) => uri,
            _ => unreachable!(),
        }
    }

    fn value_mut(full_value: &mut FullValue) -> &mut NamespaceId {
        match full_value {
            FullValue::Namespace(Namespace { uri, .. }) => uri,
            _ => unreachable!(),
        }
    }

    fn create(key: PrefixId, value: NamespaceId) -> FullValue {
        FullValue::Namespace(Namespace {
            prefix: key,
            uri: value,
        })
    }

    fn update(full_value: &mut FullValue, value: NamespaceId) -> Option<NamespaceId> {
        match full_value {
            FullValue::Namespace(Namespace { uri: old_value, .. }) => {
                let old_value = std::mem::replace(old_value, value);
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
        let attributes =
            Attributes::new(&mut xot.arena, document_element.get(), Category::Attribute);
        assert_eq!(attributes.get(&a), Some(&"A".to_string()));
    }

    #[test]
    fn test_attribute_insert() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<doc a="A"></doc>"#);
        let a = xot.add_name("a");
        let document_element = xot.document_element(root.unwrap()).unwrap();
        let mut attributes =
            Attributes::new(&mut xot.arena, document_element.get(), Category::Attribute);
        attributes.insert(a, "B".to_string());
        assert_eq!(attributes.get(&a), Some(&"B".to_string()));
    }

    #[test]
    fn test_attribute_insert_new_blank() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<doc></doc>"#);
        let a = xot.add_name("a");
        let document_element = xot.document_element(root.unwrap()).unwrap();
        let mut attributes =
            Attributes::new(&mut xot.arena, document_element.get(), Category::Attribute);
        attributes.insert(a, "A".to_string());
        assert_eq!(attributes.get(&a), Some(&"A".to_string()));
    }

    #[test]
    fn test_attribute_insert_new_existing_attributes() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<doc c="C"></doc>"#);
        let a = xot.add_name("a");
        let document_element = xot.document_element(root.unwrap()).unwrap();
        let mut attributes =
            Attributes::new(&mut xot.arena, document_element.get(), Category::Attribute);
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
        let attributes =
            Attributes::new(&mut xot.arena, document_element.get(), Category::Attribute);
        assert_eq!(attributes.get(&a), Some(&"A".to_string()));
        let namespaces =
            Namespaces::new(&mut xot.arena, document_element.get(), Category::Namespace);
        assert_eq!(namespaces.get(&foo_prefix), Some(&foo_ns));
    }
}
