use std::borrow::Cow;
use std::fmt::{Display, Formatter};

use crate::idmap::{IdIndex, IdMap};
use crate::namespace::NamespaceId;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct NameId(u16);

impl IdIndex<NameId> for NameId {
    fn to_id(index: usize) -> NameId {
        NameId(index as u16)
    }

    fn from_id(id: NameId) -> usize {
        id.0 as usize
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct Name<'a> {
    pub(crate) name: Cow<'a, str>,
    pub(crate) namespace_id: NamespaceId,
}

impl<'a> Name<'a> {
    pub(crate) fn new(name: &'a str, namespace_id: NamespaceId) -> Self {
        Self {
            name: name.into(),
            namespace_id,
        }
    }
}

pub(crate) type NameLookup<'a> = IdMap<NameId, Name<'a>>;
