// implements some of the HashMap API, based on nodes in the indextree

use ahash::AHashMap;

use crate::{xmlvalue::ValueCategory, Node, Value, Xot};

use super::entry::{Entry, OccupiedEntry, VacantEntry};

pub trait ValueAdapter<K, V> {
    fn matches(value: &Value) -> bool;
    fn children(xot: &Xot, parent: Node) -> impl Iterator<Item = Node> + '_;
    // new node insertion point is either node whether it should be inserted after,
    // or if None, prepend in the beginning
    fn insertion_point(xot: &Xot, parent: Node) -> Option<Node>;
    fn key(value: &Value) -> K;
    fn value(value: &Value) -> &V;
    fn value_mut(value: &mut Value) -> &mut V;
    fn create(key: K, value: V) -> Value;
    fn update(value: &mut Value, value: V) -> Option<V>;
}

/// A `NodeMap` is a struct with a hash-map like API and is used to
/// expose attribute and namespace prefix information.
///
/// You obtain one through the APIs [`Xot::attributes`] and [`Xot::namespaces`].
#[derive(Debug)]
pub struct NodeMap<'a, K, V, A: ValueAdapter<K, V>>
where
    K: PartialEq + Eq + Clone + Copy,
    V: Clone,
{
    xot: &'a Xot,
    parent: Node,
    _k: std::marker::PhantomData<K>,
    _v: std::marker::PhantomData<V>,
    _a: std::marker::PhantomData<A>,
}

impl<'a, K, V, A: ValueAdapter<K, V>> NodeMap<'a, K, V, A>
where
    K: PartialEq + Eq + Clone + Copy + std::hash::Hash,
    V: Clone + 'a,
{
    pub(crate) fn new(xot: &'a Xot, parent: Node) -> Self {
        Self {
            xot,
            parent,
            _k: std::marker::PhantomData,
            _v: std::marker::PhantomData,
            _a: std::marker::PhantomData,
        }
    }

    fn children(&self) -> impl Iterator<Item = Node> + '_ {
        A::children(self.xot, self.parent)
    }

    /// Get the node representing the value in the map.
    ///
    /// This is an attribute or namespace node. This node has the
    /// element node as a parent, even though it's not in `xot.children(parent)`.
    pub fn get_node(&self, key: impl Into<K> + Copy) -> Option<Node> {
        self.children()
            .find(|&child| A::key(self.xot.value(child)) == key.into())
    }

    /// Returns the number of entries in the map, also referred to as its 'length'.
    pub fn len(&self) -> usize {
        self.children().count()
    }

    /// Returns `true` if the map contains no entries.
    pub fn is_empty(&self) -> bool {
        self.children().next().is_none()
    }

    // TODO: retain, drain, sort_keys, sort_unstable_keys, sort_by, sort_unstable_by,

    /// Return `true` if an equivalent to `key` exists in the map.
    pub fn contains_key(&self, key: impl Into<K> + Copy) -> bool {
        for child in self.children() {
            if A::key(self.xot.value(child)) == key.into() {
                return true;
            }
        }
        false
    }

    /// Return a reference to the value stored for `key`, if it is present, else `None`.
    pub fn get(&self, key: impl Into<K> + Copy) -> Option<&'a V> {
        let node = self.get_node(key)?;
        Some(A::value(self.xot.value(node)))
    }

    fn iter_value(&self) -> impl Iterator<Item = &'a Value> + '_ {
        self.children().map(move |child| self.xot.value(child))
    }

    /// An iterator visiting all key-value pairs in insertion order. The iterator element type is
    /// `(&'a K, &'a V)`.
    pub fn iter(&self) -> impl Iterator<Item = (K, &'a V)> + '_ {
        self.iter_value()
            .map(|value| (A::key(value), A::value(value)))
    }

    /// Copies the map entries into a new `Vec<(K, V)>`.
    pub fn to_vec(&self) -> Vec<(K, V)> {
        self.iter().map(|(k, v)| (k, v.clone())).collect()
    }

    /// An iterator visiting all keys in insertion order. The iterator element type is `&'a K`.
    pub fn keys(&self) -> impl Iterator<Item = K> + '_ {
        self.iter_value().map(move |value| A::key(value))
    }

    /// An iterator visiting all values in insertion order. The iterator element type is `&'a V`.
    pub fn values(&self) -> impl Iterator<Item = &'a V> + '_ {
        self.iter_value().map(move |value| A::value(value))
    }

    /// An iterator visiting all the nodes in insertion order.
    pub fn nodes(&self) -> impl Iterator<Item = Node> + '_ {
        self.children()
    }

    /// Convert into a hashmap
    pub fn to_hashmap(&self) -> AHashMap<K, V> {
        let mut m = AHashMap::new();
        for (key, value) in self.iter() {
            m.insert(key, value.clone());
        }
        m
    }
}

/// A `MutableNodeMap` is a struct with a hash-map like API and is used to
/// expose attribute and namespace prefix information in a mutable way.
///
/// You obtain one through the APIs [`Xot::attributes_mut`] and
/// [`Xot::namespaces_mut`].
///
#[derive(Debug)]
pub struct MutableNodeMap<'a, K, V, A: ValueAdapter<K, V>>
where
    K: PartialEq + Eq + Clone + Copy,
    V: Clone,
{
    xot: &'a mut Xot,
    parent: Node,
    _k: std::marker::PhantomData<K>,
    _v: std::marker::PhantomData<V>,
    _a: std::marker::PhantomData<A>,
}

impl<'a, K, V, A: ValueAdapter<K, V>> MutableNodeMap<'a, K, V, A>
where
    K: PartialEq + Eq + Clone + Copy + std::hash::Hash,
    V: Clone,
{
    pub(crate) fn new(xot: &'a mut Xot, parent: Node) -> Self {
        MutableNodeMap {
            xot,
            parent,
            _k: std::marker::PhantomData,
            _v: std::marker::PhantomData,
            _a: std::marker::PhantomData,
        }
    }

    // TODO argh duplication
    fn children(&self) -> impl Iterator<Item = Node> + '_ {
        A::children(self.xot, self.parent)
    }

    /// Get the node representing the value in the map.
    ///
    /// This is an attribute or namespace node. This node has the
    /// element node as a parent, even though it's not in `xot.children(parent)`.
    pub fn get_node(&self, key: impl Into<K> + Copy) -> Option<Node> {
        self.children()
            .find(|&child| A::key(self.xot.value(child)) == key.into())
    }

    /// Returns the number of entries in the map, also referred to as its 'length'.
    pub fn len(&self) -> usize {
        self.children().count()
    }

    /// Returns `true` if the map contains no entries.
    pub fn is_empty(&self) -> bool {
        self.children().next().is_some()
    }

    // TODO: retain, drain, sort_keys, sort_unstable_keys, sort_by, sort_unstable_by,

    /// Return `true` if an equivalent to `key` exists in the map.
    pub fn contains_key(&self, key: impl Into<K> + Copy) -> bool {
        for child in self.children() {
            if A::key(self.xot.value(child)) == key.into() {
                return true;
            }
        }
        false
    }

    /// Return a reference to the value stored for `key`, if it is present, else `None`.
    pub fn get(&self, key: impl Into<K> + Copy) -> Option<&V> {
        let node = self.get_node(key)?;
        Some(A::value(self.xot.value(node)))
    }

    fn iter_value(&'a self) -> impl Iterator<Item = &'a Value> + '_ {
        self.children().map(move |child| self.xot.value(child))
    }

    /// An iterator visiting all key-value pairs in insertion order. The iterator element type is
    /// `(&'a K, &'a V)`.
    pub fn iter(&'a self) -> impl Iterator<Item = (K, &'a V)> + '_ {
        self.iter_value()
            .map(move |value| (A::key(value), A::value(value)))
    }

    /// Copies the map entries into a new `Vec<(K, V)>`.
    pub fn to_vec(&self) -> Vec<(K, V)> {
        self.iter().map(|(k, v)| (k, v.clone())).collect()
    }

    /// An iterator visiting all keys in insertion order. The iterator element type is `&'a K`.
    pub fn keys(&'a self) -> impl Iterator<Item = K> + '_ {
        self.iter_value().map(move |value| A::key(value))
    }

    /// An iterator visiting all values in insertion order. The iterator element type is `&'a V`.
    pub fn values(&'a self) -> impl Iterator<Item = &'a V> + '_ {
        self.iter_value().map(move |value| A::value(value))
    }

    /// An iterator visiting all the nodes in insertion order.
    pub fn nodes(&self) -> impl Iterator<Item = Node> + '_ {
        self.children()
    }

    /// Convert into a hashmap.
    pub fn to_hashmap(&self) -> AHashMap<K, V> {
        let mut m = AHashMap::new();
        for (key, value) in self.iter() {
            m.insert(key, value.clone());
        }
        m
    }

    // TODO: end of duplication

    /// Return a mutable reference to the value stored for `key`, if it is present, else `None`.
    pub fn get_mut(&mut self, key: impl Into<K> + Copy) -> Option<&mut V> {
        let node = self.get_node(key)?;
        Some(A::value_mut(self.xot.value_mut(node)))
    }

    /// Clears the map, removing all entries.
    pub fn clear(&mut self) {
        let to_remove = self.children().collect::<Vec<_>>();

        for child in to_remove {
            self.xot.remove(child).unwrap();
        }
    }

    /// Insert a key-value pair in the map.
    ///
    /// If an equivalent key already exists in the map: the key remains and retains in its place
    /// in the order, its corresponding value is updated with `value` and the older value is
    /// returned inside `Some(_)`.
    ///
    /// If no equivalent key existed in the map: the new key-value pair is inserted, last in
    /// order, and `None` is returned.
    ///
    /// See also [`entry`](#method.entry) if you you want to insert *or* modify or if you need to
    /// get the index of the corresponding key-value pair.
    pub fn insert(&mut self, key: impl Into<K> + Copy, value: V) -> Option<V> {
        let node = self.get_node(key);
        if let Some(node) = node {
            // if we already have a node
            let node_value = self.xot.value_mut(node);
            A::update(node_value, value)
        } else {
            // we need to insert a new node
            let new_value = A::create(key.into(), value);
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

    /// Insert a node into the map. This node has to be of the right type,
    /// so a namespace node for a namespaces map, and an attribute node
    /// for an attribute map, if not, this function will panic.
    ///
    /// If an equivalent key already exists in the map: existing node is updated
    /// with the new value, and returned.
    ///
    /// If no equivalent key existed in the map: the new node is inserted, and
    /// returned as the inserted node.
    pub(crate) fn insert_node(&mut self, node: Node) -> Node {
        let node_value = self.xot.value(node);
        if !A::matches(node_value) {
            panic!("Tried to insert unexpected node value into the node map");
        }
        let key = A::key(node_value);
        let value = A::value(node_value).clone();

        let existing_node = self.get_node(key);
        if let Some(existing_node) = existing_node {
            // if we already have a node
            let node_value = self.xot.value_mut(existing_node);
            A::update(node_value, value);
            existing_node
        } else {
            let insertion_point = A::insertion_point(self.xot, self.parent);
            if let Some(insertion_point) = insertion_point {
                insertion_point
                    .get()
                    .checked_insert_after(node.get(), &mut self.xot.arena)
                    .unwrap();
            } else {
                self.parent
                    .get()
                    .checked_prepend(node.get(), &mut self.xot.arena)
                    .unwrap();
            }
            node
        }
    }

    /// Remove a key-value pair from the map, if it exists.
    ///
    /// Returns the value corresponding to the key if the key was previously in the map.
    pub fn remove(&mut self, key: impl Into<K> + Copy) -> Option<V> {
        let node = self.get_node(key);
        if let Some(node) = node {
            let value = A::value(self.xot.value(node)).clone();
            self.xot.remove(node).unwrap();
            Some(value)
        } else {
            None
        }
    }

    /// Get the given key's corresponding entry in the map for insertion and/or in-place
    /// manipulation.
    pub fn entry(&'a mut self, key: impl Into<K> + Copy) -> Entry<'a, K, V, A> {
        match self.get(key) {
            Some(_value) => Entry::Occupied(OccupiedEntry::new(self, key.into())),
            None => Entry::Vacant(VacantEntry::new(self, key.into())),
        }
    }
}

pub(crate) fn category_predicate(
    xot: &Xot,
    category: ValueCategory,
) -> impl Fn(&Node) -> bool + '_ {
    move |node| xot.value(*node).value_category() == category
}

#[cfg(test)]
mod tests {
    use crate::Xot;

    #[test]
    fn test_attribute_get() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<doc a="A"></doc>"#);
        let a = xot.add_name("a");
        let document_element = xot.document_element(root.unwrap()).unwrap();
        let attributes = xot.attributes(document_element);
        assert_eq!(attributes.get(a), Some(&"A".to_string()));
    }

    #[test]
    fn test_attribute_insert() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<doc a="A"></doc>"#).unwrap();
        let a = xot.add_name("a");
        let document_element = xot.document_element(root).unwrap();
        let mut attributes = xot.attributes_mut(document_element);
        attributes.insert(a, "B".to_string());
        assert_eq!(attributes.get(a), Some(&"B".to_string()));
        assert_eq!(xot.to_string(root).unwrap(), r#"<doc a="B"/>"#);
    }

    #[test]
    fn test_attribute_insert_node() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<doc a="A"></doc>"#).unwrap();
        let a = xot.add_name("a");
        let document_element = xot.document_element(root).unwrap();
        let a_node = xot.new_attribute_node(a, "B".to_string());
        let mut attributes = xot.attributes_mut(document_element);
        let inserted_node = attributes.insert_node(a_node);
        assert_ne!(inserted_node, a_node);
        assert_eq!(attributes.get(a), Some(&"B".to_string()));
        assert_eq!(xot.to_string(root).unwrap(), r#"<doc a="B"/>"#);
    }

    #[test]
    fn test_attribute_insert_new_blank() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<doc></doc>"#).unwrap();
        let a = xot.add_name("a");
        let document_element = xot.document_element(root).unwrap();
        let mut attributes = xot.attributes_mut(document_element);
        attributes.insert(a, "A".to_string());
        assert_eq!(attributes.get(a), Some(&"A".to_string()));
        assert_eq!(xot.to_string(root).unwrap(), r#"<doc a="A"/>"#);
    }

    #[test]
    fn test_attribute_insert_node_new_blank() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<doc></doc>"#).unwrap();
        let a = xot.add_name("a");
        let document_element = xot.document_element(root).unwrap();
        let a_node = xot.new_attribute_node(a, "A".to_string());
        let mut attributes = xot.attributes_mut(document_element);
        let inserted_node = attributes.insert_node(a_node);
        assert_eq!(inserted_node, a_node);
        assert_eq!(attributes.get(a), Some(&"A".to_string()));
        assert_eq!(xot.to_string(root).unwrap(), r#"<doc a="A"/>"#);
    }

    #[test]
    fn test_attribute_insert_new_existing_attributes() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<doc c="C"></doc>"#).unwrap();
        let a = xot.add_name("a");
        let document_element = xot.document_element(root).unwrap();
        let mut attributes = xot.attributes_mut(document_element);
        attributes.insert(a, "A".to_string());
        assert_eq!(attributes.get(a), Some(&"A".to_string()));
        assert_eq!(xot.to_string(root).unwrap(), r#"<doc c="C" a="A"/>"#);
    }

    #[test]
    fn test_attribute_insert_node_new_existing_attributes() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<doc c="C"></doc>"#).unwrap();
        let a = xot.add_name("a");
        let document_element = xot.document_element(root).unwrap();
        let a_node = xot.new_attribute_node(a, "A".to_string());
        let mut attributes = xot.attributes_mut(document_element);
        let inserted_node = attributes.insert_node(a_node);
        assert_eq!(inserted_node, a_node);
        assert_eq!(attributes.get(a), Some(&"A".to_string()));
        assert_eq!(xot.to_string(root).unwrap(), r#"<doc c="C" a="A"/>"#);
    }

    #[test]
    fn test_attribute_entry_modify() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<doc c="C"></doc>"#);
        let c = xot.add_name("c");
        let document_element = xot.document_element(root.unwrap()).unwrap();
        let mut attributes = xot.attributes_mut(document_element);
        attributes
            .entry(c)
            .and_modify(|e| *e = "C!".to_string())
            .or_insert("New".to_string());
        let attributes = xot.attributes(document_element);
        assert_eq!(attributes.get(c), Some(&"C!".to_string()));
    }

    #[test]
    fn test_attribute_entry_create() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<doc></doc>"#);
        let c = xot.add_name("c");
        let document_element = xot.document_element(root.unwrap()).unwrap();
        let mut attributes = xot.attributes_mut(document_element);
        attributes
            .entry(c)
            .and_modify(|e| *e = "C!".to_string())
            .or_insert("New".to_string());
        let attributes = xot.attributes(document_element);
        assert_eq!(attributes.get(c), Some(&"New".to_string()));
    }

    #[test]
    fn test_attributes_and_namespaces() {
        let mut xot = Xot::new();

        let root = xot.parse(r#"<doc xmlns:foo="FOO" a="A"></doc>"#);
        let a = xot.add_name("a");
        let foo_prefix = xot.add_prefix("foo");
        let foo_ns = xot.add_namespace("FOO");
        let document_element = xot.document_element(root.unwrap()).unwrap();
        let attributes = xot.attributes(document_element);
        assert_eq!(attributes.get(a), Some(&"A".to_string()));
        let namespaces = xot.namespaces(document_element);
        assert_eq!(namespaces.get(foo_prefix), Some(&foo_ns));
    }
}
