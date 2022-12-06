use std::fmt::{Display, Formatter};

use crate::idmap::{IdIndex, IdMap};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct PrefixId(u16);

impl IdIndex<PrefixId> for PrefixId {
    fn to_id(index: usize) -> PrefixId {
        PrefixId(index as u16)
    }

    fn from_id(id: PrefixId) -> usize {
        id.0 as usize
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct Prefix(String);

impl Prefix {
    pub(crate) fn new(prefix: String) -> Self {
        Self(prefix)
    }
}

impl Display for Prefix {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub(crate) type PrefixLookup = IdMap<PrefixId, Prefix>;
