use std::fmt::{Display, Formatter};

use crate::idmap::{IdIndex, IdMap};

/// Id uniquely identifying namespace.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct NamespaceId(u16);

impl IdIndex<NamespaceId> for NamespaceId {
    fn to_id(index: usize) -> NamespaceId {
        NamespaceId(index as u16)
    }

    fn from_id(id: NamespaceId) -> usize {
        id.0 as usize
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Namespace(String);

impl Namespace {
    pub(crate) fn new(namespace_uri: String) -> Self {
        Self(namespace_uri)
    }

    pub(crate) fn get(&self) -> &str {
        &self.0
    }
}

impl Display for Namespace {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub(crate) type NamespaceLookup = IdMap<NamespaceId, Namespace>;
