use crate::idmap::{IdIndex, IdMap};

/// Id uniquely identifying a prefix
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct PrefixId(u16);

impl IdIndex<PrefixId> for PrefixId {
    fn to_id(index: usize) -> PrefixId {
        PrefixId(index as u16)
    }

    fn from_id(id: PrefixId) -> usize {
        id.0 as usize
    }
}

pub(crate) type PrefixLookup = IdMap<PrefixId, String>;
