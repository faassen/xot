use super::core::{MutableNodeMap, ValueAdapter};

/// Entry for an existing key-value pair or a vacant location to insert one.
#[derive(Debug)]
pub enum Entry<'a, K, V, A: ValueAdapter<K, V>>
where
    K: PartialEq + Eq + Clone + Copy,
    V: Clone,
{
    /// Occupied entry.
    Occupied(OccupiedEntry<'a, K, V, A>),
    /// Vacant entry.
    Vacant(VacantEntry<'a, K, V, A>),
}

impl<'a, K, V, A: ValueAdapter<K, V>> Entry<'a, K, V, A>
where
    K: PartialEq + Eq + Clone + Copy,
    V: Clone,
{
    /// Ensures a value is in the entry by inserting the default if empty, and returns a mutable
    /// reference to the value in the entry.
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default function if empty, and
    /// returns a mutable reference to the value in the entry.
    pub fn or_insert_with<F: FnOnce() -> V>(self, call: F) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(call()),
        }
    }

    /// Returns a reference to this entry's key.
    pub fn key(&self) -> &K {
        match self {
            Entry::Occupied(entry) => entry.key(),
            Entry::Vacant(entry) => entry.key(),
        }
    }

    /// Provides in-place mutable access to an occupied entry before any potential inserts into the
    /// map.
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        match self {
            Entry::Occupied(mut entry) => {
                f(entry.get_mut());
                Entry::Occupied(entry)
            }
            Entry::Vacant(entry) => Entry::Vacant(entry),
        }
    }

    /// Ensures a value is in the entry by inserting the default value if empty,
    /// and returns a mutable reference to the value in the entry.
    pub fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(V::default()),
        }
    }
}

/// A view into an occupied entry in a `NodeMap`. It is part of the [`Entry`] enum.
#[derive(Debug)]
pub struct OccupiedEntry<'a, K, V, A: ValueAdapter<K, V>>
where
    K: PartialEq + Eq + Clone + Copy,
    V: Clone,
{
    map: &'a mut MutableNodeMap<'a, K, V, A>,
    key: K,
}

impl<'a, K, V, A> OccupiedEntry<'a, K, V, A>
where
    K: PartialEq + Eq + Clone + Copy,
    V: Clone,
    A: ValueAdapter<K, V>,
{
    pub(crate) fn new(map: &'a mut MutableNodeMap<'a, K, V, A>, key: K) -> Self {
        Self { map, key }
    }

    /// Gets a reference to the value in the entry.
    pub fn get(&self) -> &V {
        self.map.get(self.key).unwrap()
    }

    /// Gets a mutable reference to the value in the entry.
    pub fn get_mut(&mut self) -> &mut V {
        self.map.get_mut(self.key).unwrap()
    }

    /// Converts the entry into a mutable reference to the value in the entry with a lifetime bound
    /// to the map itself.
    pub fn into_mut(self) -> &'a mut V {
        self.map.get_mut(self.key).unwrap()
    }

    /// Sets the value of the entry, and returns the entry's old value.
    pub fn insert(&mut self, value: V) -> V {
        self.map.insert(self.key, value).unwrap()
    }

    /// Takes the value of the entry out of the map, and returns it.
    pub fn remove(self) -> V {
        self.map.remove(self.key).unwrap()
    }

    /// Gets a reference to the key in the entry.
    pub fn key(&self) -> &K {
        &self.key
    }
}

/// A view into a vacant entry in a `NodeMap`. It is part of the [`Entry`] enum.
#[derive(Debug)]
pub struct VacantEntry<'a, K, V, A: ValueAdapter<K, V>>
where
    K: PartialEq + Eq + Clone + Copy,
    V: Clone,
{
    map: &'a mut MutableNodeMap<'a, K, V, A>,
    key: K,
}

impl<'a, K, V, A> VacantEntry<'a, K, V, A>
where
    K: PartialEq + Eq + Clone + Copy,
    V: Clone,
    A: ValueAdapter<K, V>,
{
    pub(crate) fn new(map: &'a mut MutableNodeMap<'a, K, V, A>, key: K) -> Self {
        Self { map, key }
    }

    /// Sets the value of the entry with the VacantEntry's key, and returns a mutable reference to it.
    pub fn insert(self, value: V) -> &'a mut V {
        self.map.insert(self.key, value).unwrap();
        self.map.get_mut(self.key).unwrap()
    }

    /// Gets a reference to the key that would be used when inserting a value through the VacantEntry.
    pub fn key(&self) -> &K {
        &self.key
    }
}
