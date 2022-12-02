use std::borrow::Cow;

use crate::idmap::{IdIndex, IdMap};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct NamespaceId(u8);

impl IdIndex<NamespaceId> for NamespaceId {
    fn to_id(index: usize) -> NamespaceId {
        NamespaceId(index as u8)
    }

    fn from_id(id: NamespaceId) -> usize {
        id.0 as usize
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Namespace<'a>(Cow<'a, str>);

impl<'a> Namespace<'a> {
    pub(crate) fn new(namespace_uri: &'a str) -> Self {
        Self(namespace_uri.into())
    }
}

pub(crate) type NamespaceLookup<'a> = IdMap<NamespaceId, Namespace<'a>>;
