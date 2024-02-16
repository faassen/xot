// An efficient way to store XML name information as part of state.

use crate::{
    id::{NameId, NamespaceId, PrefixId},
    Error, Xot,
};

use super::{
    reference::{Lookup, NameIdInfo},
    XmlNameRef,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct XmlNameState {
    name_id: NameId,
    namespace_id: NamespaceId,
    prefix_id: PrefixId,
}

impl NameIdInfo for XmlNameState {
    fn name_id(&self) -> NameId {
        self.name_id
    }

    fn namespace_id(&self) -> NamespaceId {
        self.namespace_id
    }

    fn prefix_id(&self) -> Result<PrefixId, Error> {
        Ok(self.prefix_id)
    }
}

// we don't actually need to look up anything for the state version
struct NullLookup;

impl Lookup for NullLookup {
    fn prefix_id_for_namespace_id(&self, _namespace_id: NamespaceId) -> Option<PrefixId> {
        None
    }

    fn namespace_id_for_prefix_id(&self, _prefix_id: PrefixId) -> Option<NamespaceId> {
        None
    }
}

impl XmlNameState {
    pub(crate) fn new(name_id: NameId, namespace_id: NamespaceId, prefix_id: PrefixId) -> Self {
        Self {
            name_id,
            namespace_id,
            prefix_id,
        }
    }

    fn into_ref(self, xot: &Xot) -> XmlNameRef<NullLookup> {
        XmlNameRef::new(xot, NullLookup, self.name_id)
    }
}
