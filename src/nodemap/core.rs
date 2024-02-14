// implements some of the HashMap API, based on nodes in the indextree

use crate::{xmlvalue::ValueCategory, Node, Value, Xot};

use super::entry::{Entry, OccupiedEntry, VacantEntry};

pub trait ValueAdapter<K, V> {
    fn children(xot: &Xot, parent: Node) -> impl Iterator<Item = Node> + '_;
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
    K: PartialEq + Eq + Clone + Copy,
    V: Clone,
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

    fn get_node_q(&self, key: &K) -> Option<Node> {
        self.children()
            .find(|&child| A::key(self.xot.value(child)) == key)
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
    pub fn contains_key(&self, key: &K) -> bool {
        for child in self.children() {
            if A::key(self.xot.value(child)) == key {
                return true;
            }
        }
        false
    }

    /// Return a reference to the value stored for `key`, if it is present, else `None`.
    pub fn get(&self, key: &K) -> Option<&V> {
        let node = self.get_node_q(key)?;
        Some(A::value(self.xot.value(node)))
    }

    fn iter_value(&self) -> impl Iterator<Item = &'a Value> + '_ {
        self.children().map(move |child| self.xot.value(child))
    }

    /// An iterator visiting all key-value pairs in insertion order. The iterator element type is
    /// `(&'a K, &'a V)`.
    pub fn iter(&self) -> impl Iterator<Item = (&'a K, &'a V)> + '_ {
        self.iter_value()
            .map(|value| (A::key(value), A::value(value)))
    }

    /// Copies the map entries into a new `Vec<(K, V)>`.
    pub fn to_vec(&self) -> Vec<(K, V)> {
        self.iter().map(|(k, v)| (*k, v.clone())).collect()
    }

    /// An iterator visiting all keys in insertion order. The iterator element type is `&'a K`.
    pub fn keys(&self) -> impl Iterator<Item = &'a K> + '_ {
        self.iter_value().map(move |value| A::key(value))
    }

    /// An iterator visiting all values in insertion order. The iterator element type is `&'a V`.
    pub fn values(&self) -> impl Iterator<Item = &'a V> + '_ {
        self.iter_value().map(move |value| A::value(value))
    }
}

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
    K: PartialEq + Eq + Clone + Copy,
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

    fn get_node_q(&self, key: &K) -> Option<Node> {
        self.children()
            .find(|&child| A::key(self.xot.value(child)) == key)
    }

    fn get_node(&self, key: &K) -> Option<Node> {
        self.children()
            .find(|&child| A::key(self.xot.value(child)) == key)
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
    pub fn contains_key(&self, key: &K) -> bool {
        for child in self.children() {
            if A::key(self.xot.value(child)) == key {
                return true;
            }
        }
        false
    }

    /// Return a reference to the value stored for `key`, if it is present, else `None`.
    pub fn get(&self, key: &K) -> Option<&V> {
        let node = self.get_node_q(key)?;
        Some(A::value(self.xot.value(node)))
    }

    fn iter_value(&'a self) -> impl Iterator<Item = &'a Value> + '_ {
        self.children().map(move |child| self.xot.value(child))
    }

    /// An iterator visiting all key-value pairs in insertion order. The iterator element type is
    /// `(&'a K, &'a V)`.
    pub fn iter(&'a self) -> impl Iterator<Item = (&'a K, &'a V)> + '_ {
        self.iter_value()
            .map(move |value| (A::key(value), A::value(value)))
    }

    /// Copies the map entries into a new `Vec<(K, V)>`.
    pub fn to_vec(&self) -> Vec<(K, V)> {
        self.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// An iterator visiting all keys in insertion order. The iterator element type is `&'a K`.
    pub fn keys(&'a self) -> impl Iterator<Item = &'a K> + '_ {
        self.iter_value().map(move |value| A::key(value))
    }

    /// An iterator visiting all values in insertion order. The iterator element type is `&'a V`.
    pub fn values(&'a self) -> impl Iterator<Item = &'a V> + '_ {
        self.iter_value().map(move |value| A::value(value))
    }

    // TODO: end of duplication

    /// Return a mutable reference to the value stored for `key`, if it is present, else `None`.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let node = self.get_node_q(key)?;
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

    pub fn remove(&mut self, key: &K) -> Option<V> {
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
    pub fn entry(&'a mut self, key: K) -> Entry<'a, K, V, A> {
        match self.get(&key) {
            Some(_value) => Entry::Occupied(OccupiedEntry::new(self, key)),
            None => Entry::Vacant(VacantEntry::new(self, key)),
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
        assert_eq!(attributes.get(&a), Some(&"A".to_string()));
    }

    #[test]
    fn test_attribute_insert() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<doc a="A"></doc>"#);
        let a = xot.add_name("a");
        let document_element = xot.document_element(root.unwrap()).unwrap();
        let mut attributes = xot.attributes_mut(document_element);
        attributes.insert(a, "B".to_string());
        assert_eq!(attributes.get(&a), Some(&"B".to_string()));
    }

    #[test]
    fn test_attribute_insert_new_blank() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<doc></doc>"#);
        let a = xot.add_name("a");
        let document_element = xot.document_element(root.unwrap()).unwrap();
        let mut attributes = xot.attributes_mut(document_element);
        attributes.insert(a, "A".to_string());
        assert_eq!(attributes.get(&a), Some(&"A".to_string()));
    }

    #[test]
    fn test_attribute_insert_new_existing_attributes() {
        let mut xot = Xot::new();
        let root = xot.parse(r#"<doc c="C"></doc>"#);
        let a = xot.add_name("a");
        let document_element = xot.document_element(root.unwrap()).unwrap();
        let mut attributes = xot.attributes_mut(document_element);
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
        let attributes = xot.attributes(document_element);
        assert_eq!(attributes.get(&a), Some(&"A".to_string()));
        let namespaces = xot.namespaces(document_element);
        assert_eq!(namespaces.get(&foo_prefix), Some(&foo_ns));
    }
}
