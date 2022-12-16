use ahash::HashMap;
use std::borrow::Borrow;
use std::hash::Hash;

pub(crate) trait IdIndex<T> {
    fn to_id(index: usize) -> T;
    fn from_id(id: T) -> usize;
}

pub(crate) struct IdMap<K: Copy + IdIndex<K>, V: Eq + std::hash::Hash + Clone> {
    by_id: Vec<V>,
    by_value: HashMap<V, K>,
}

impl<K: Copy + IdIndex<K>, V: Eq + std::hash::Hash + Clone> IdMap<K, V> {
    pub(crate) fn new() -> Self {
        IdMap {
            by_id: Vec::new(),
            by_value: HashMap::default(),
        }
    }

    pub(crate) fn get_id_mut<Q: ?Sized>(&mut self, value: &Q) -> K
    where
        V: Borrow<Q>,
        Q: Hash + Eq + ToOwned<Owned = V>,
    {
        let id = self.by_value.get(value);
        if let Some(id) = id {
            *id
        } else {
            let id = K::to_id(self.by_id.len());
            let cloned = value.to_owned();
            self.by_value.insert(cloned.clone(), id);
            self.by_id.push(cloned);
            id
        }
    }

    pub(crate) fn get_id<Q: ?Sized>(&self, value: &Q) -> Option<K>
    where
        V: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.by_value.get(value).copied()
    }

    #[inline]
    pub(crate) fn get_value(&self, id: K) -> &V {
        &self.by_id[K::from_id(id)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_map() {
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        struct Id(u32);

        impl IdIndex<Id> for Id {
            fn to_id(index: usize) -> Id {
                Id(index as u32)
            }

            fn from_id(id: Id) -> usize {
                id.0 as usize
            }
        }

        let mut map = IdMap::<Id, String>::new();
        let id1 = map.get_id_mut("foo");
        let id2 = map.get_id_mut("bar");
        let id3 = map.get_id_mut("foo");
        assert_eq!(id1, id3);
        assert_ne!(id1, id2);
        assert_eq!(map.get_value(id1), &"foo");
        assert_eq!(map.get_value(id2), &"bar");

        let id4 = map.get_id("foo").unwrap();
        assert_eq!(id1, id4);
    }

    // #[test]
    // fn test_id_map_with_cow() {
    //     use std::borrow::Cow;

    //     #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    //     struct Id(u32);

    //     impl IdIndex<Id> for Id {
    //         fn to_id(index: usize) -> Id {
    //             Id(index as u32)
    //         }

    //         fn from_id(id: Id) -> usize {
    //             id.0 as usize
    //         }
    //     }

    //     let mut map = IdMap::<Id, Cow<'static, str>>::new();
    //     let id1 = map.get_id_mut(Cow::Borrowed("foo"));
    //     let id2 = map.get_id_mut(Cow::Borrowed("bar"));
    //     let id3 = map.get_id_mut(Cow::Borrowed("foo"));
    //     assert_eq!(id1, id3);
    //     assert_ne!(id1, id2);
    //     assert_eq!(map.get_value(id1), "foo");
    //     assert_eq!(map.get_value(id2), "bar");
    // }
}
