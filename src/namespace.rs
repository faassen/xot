use std::borrow::Cow;
use std::fmt::{Display, Formatter};

use crate::idmap::{IdIndex, IdMap};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
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
pub struct Namespace<'a>(Cow<'a, str>);

impl<'a> Namespace<'a> {
    pub(crate) fn new(namespace_uri: Cow<'a, str>) -> Self {
        Self(namespace_uri)
    }
}

impl<'a> Display for Namespace<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub(crate) type NamespaceLookup<'a> = IdMap<NamespaceId, Namespace<'a>>;
