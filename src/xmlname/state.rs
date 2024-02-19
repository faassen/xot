use crate::{
    id::{NameId, NamespaceId, PrefixId},
    Xot,
};

use super::{owned::OwnedName, reference::NameIdInfo, RefName};

/// This is an efficient way to store name information as application state.
///
/// There are no direct references to Xot, so you need to provide a [`Xot`]` to
/// convert it back to a [`RefName`] using [`State.to_ref`].
///
/// It supports id access using the [`NameIdInfo`] trait.
///
/// It can also be used to create new elements and attributes, just like a
/// [`crate::NameId`].
#[derive(Debug, Clone, Copy)]
pub struct StateName {
    name_id: NameId,
    // this is redundant with the name id, but we don't want to have to
    // do a xot lookup to get the namespace id
    namespace_id: NamespaceId,
    prefix_id: PrefixId,
}

impl std::hash::Hash for StateName {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name_id.hash(state);
    }
}

impl PartialEq for StateName {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.name_id == other.name_id
    }
}

impl Eq for StateName {}

impl NameIdInfo for StateName {
    #[inline]
    fn name_id(&self) -> NameId {
        self.name_id
    }

    #[inline]
    fn namespace_id(&self) -> NamespaceId {
        self.namespace_id
    }

    #[inline]
    fn prefix_id(&self) -> PrefixId {
        self.prefix_id
    }
}

impl StateName {
    pub(crate) fn new(name_id: NameId, namespace_id: NamespaceId, prefix_id: PrefixId) -> Self {
        Self {
            name_id,
            namespace_id,
            prefix_id,
        }
    }

    /// Create a new [`RefName`] from this state.
    ///
    /// This is an efficient way to access its name string information.
    #[inline]
    pub fn to_ref(self, xot: &Xot) -> RefName {
        RefName::new(xot, self.name_id, self.prefix_id)
    }

    /// Create a new [`OwnedName`] from this state
    ///
    /// If you want to access name information it's more efficient to create a
    /// reference with [`StateName::to_ref`] and then use the accessors.
    pub fn to_owned(self, xot: &Xot) -> OwnedName {
        self.to_ref(xot).to_owned()
    }
}
