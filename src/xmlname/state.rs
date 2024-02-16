use crate::{
    id::{NameId, NamespaceId, PrefixId},
    Error, Xot,
};

use super::{
    owned::XmlNameOwned,
    reference::{Lookup, NameIdInfo},
    XmlNameRef,
};

/// This is an efficient way to store name information.
///
/// You don't need to worry about references to store this information.
///
/// It supports id access using the [`NameIdInfo`] trait.
///
/// To access the string information again, you can use [`XmlNameState::to_ref`]
#[derive(Debug, Clone, Copy)]
pub struct XmlNameState {
    name_id: NameId,
    // this is redundant with the name id, but we don't want to have to
    // do a xot lookup to get the namespace id
    namespace_id: NamespaceId,
    prefix_id: PrefixId,
}

impl std::hash::Hash for XmlNameState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name_id.hash(state);
    }
}

impl PartialEq for XmlNameState {
    fn eq(&self, other: &Self) -> bool {
        self.name_id == other.name_id
    }
}

impl Eq for XmlNameState {}

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
        unreachable!()
    }

    fn namespace_id_for_prefix_id(&self, _prefix_id: PrefixId) -> Option<NamespaceId> {
        unreachable!()
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

    /// Create a new [`crate::xmlname::XmlNameRef`] from this state.
    ///
    /// This is an efficient way to access its name string information.
    fn to_ref(self, xot: &Xot) -> XmlNameRef<NullLookup> {
        XmlNameRef::new(xot, NullLookup, self.name_id)
    }

    /// Create a new [`crate::xmlname::XmlNameOwned`] from this state
    ///
    /// If you want to access name information it's more efficient to create
    /// a reference with [`XmlNameState::to_ref`] and then use the accessors.
    fn to_owned(self, xot: &Xot) -> Result<XmlNameOwned, Error> {
        self.to_ref(xot).to_owned()
    }
}
