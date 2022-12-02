use ahash::HashMap;

trait IdIndex<T> {
    fn to_id(index: usize) -> T;
    fn from_id(id: T) -> usize;
}

struct IdMap<K: Copy, V: Eq + std::hash::Hash + Clone, I: IdIndex<K>> {
    by_id: Vec<V>,
    by_value: HashMap<V, K>,
    p: std::marker::PhantomData<I>,
}

impl<K: Copy, V: Eq + std::hash::Hash + Clone, I: IdIndex<K>> IdMap<K, V, I> {
    pub(crate) fn new() -> Self {
        IdMap {
            by_id: Vec::new(),
            by_value: HashMap::default(),
            p: std::marker::PhantomData,
        }
    }

    pub(crate) fn get_id(&mut self, value: V) -> K {
        let id = self.by_value.get(&value);
        if let Some(id) = id {
            *id
        } else {
            let id = I::to_id(self.by_id.len());
            let cloned = value.clone();
            self.by_value.insert(cloned, id);
            self.by_id.push(value);
            id
        }
    }

    #[inline]
    pub(crate) fn get_value(&self, id: K) -> &V {
        &self.by_id[I::from_id(id)]
    }
}
