use crate::{
    id::{NameId, NamespaceId, PrefixId},
    Error, Xot,
};

use super::{
    owned::Owned,
    reference::{Lookup, NameIdInfo},
    Ref,
};

/// This is an efficient way to store name information.
///
/// There are no direct references to Xot, so you need to provide Xot
/// to convert it back to a [`Ref`] using [`State.to_ref`].
///
/// It supports id access using the [`NameIdInfo`] trait.
///
/// It can also be used to create new elements and attributes.
#[derive(Debug, Clone, Copy)]
pub struct State {
    name_id: NameId,
    // this is redundant with the name id, but we don't want to have to
    // do a xot lookup to get the namespace id
    namespace_id: NamespaceId,
    prefix_id: PrefixId,
}

impl std::hash::Hash for State {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name_id.hash(state);
    }
}

impl PartialEq for State {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.name_id == other.name_id
    }
}

impl Eq for State {}

impl NameIdInfo for State {
    #[inline]
    fn name_id(&self) -> NameId {
        self.name_id
    }

    #[inline]
    fn namespace_id(&self) -> NamespaceId {
        self.namespace_id
    }

    #[inline]
    fn prefix_id(&self) -> Result<PrefixId, Error> {
        Ok(self.prefix_id)
    }
}

// we don't actually need to look up anything for the state version
#[derive(Debug, Clone, Copy)]
pub struct NullLookup;

impl Lookup for NullLookup {
    fn prefix_id_for_namespace_id(&self, _namespace_id: NamespaceId) -> Option<PrefixId> {
        unreachable!()
    }
}

impl State {
    pub(crate) fn new(name_id: NameId, namespace_id: NamespaceId, prefix_id: PrefixId) -> Self {
        Self {
            name_id,
            namespace_id,
            prefix_id,
        }
    }

    /// Create a new [`Ref`] from this state.
    ///
    /// This is an efficient way to access its name string information.
    #[inline]
    pub fn to_ref(self, xot: &Xot) -> Ref<NullLookup> {
        Ref::new(xot, NullLookup, self.name_id)
    }

    /// Create a new [`Owned`] from this state
    ///
    /// If you want to access name information it's more efficient to create
    /// a reference with [`State::to_ref`] and then use the accessors.
    pub fn to_owned(self, xot: &Xot) -> Result<Owned, Error> {
        self.to_ref(xot).to_owned()
    }
}
