use std::borrow::Cow;

use crate::id::{NameId, NamespaceId, PrefixId};
use crate::xotdata::Xot;
use crate::{Error, Node};

use super::owned::Owned;
use super::state::State;

/// Name id information.
///
/// Name ids are backed by Xot and given a Xot reference can be turned
/// back into strings.
pub trait NameIdInfo {
    /// Access the underlying name id
    fn name_id(&self) -> NameId;

    /// Access the underlying namespace id
    fn namespace_id(&self) -> NamespaceId;

    /// Get the prefix for the name in the context of a node.
    ///
    /// If this is in the default namespace, this is the empty string.
    ///
    /// If the prefix cannot be found, return an [`Error::MissingPrefix`].
    fn prefix_id(&self) -> Result<PrefixId, Error>;
}

impl<N: NameIdInfo> From<N> for NameId {
    fn from(name: N) -> Self {
        name.name_id()
    }
}

/// Name string information for an xml name.
pub trait NameStrInfo {
    /// Access the local name as a string reference
    fn local_name(&self) -> &str;

    /// Get the namespace uri as a str reference.
    ///
    /// If there is no namespace, this is the empty string.
    fn namespace(&self) -> &str;

    /// Access the prefix as a string
    fn prefix(&self) -> Result<&str, Error>;

    /// Access the full name as a string
    fn full_name(&self) -> Result<Cow<str>, Error> {
        let prefix = self.prefix()?;
        if !prefix.is_empty() {
            Ok(Cow::Owned(format!("{}:{}", prefix, self.local_name())))
        } else {
            Ok(Cow::Borrowed(self.local_name()))
        }
    }
}

/// Lookup of prefix information given some context.
///
/// There are various ways to determine the prefix for a namespace id; sometimes
/// the prefix may already be known, sometimes it needs to be looked up in the context
/// of a node.
pub trait Lookup {
    /// Look up a prefix id given a namespace id.
    fn prefix_id_for_namespace_id(&self, namespace_id: NamespaceId) -> Option<PrefixId>;
}

/// The most complete way to access name information, backed by Xot. This is a
/// reference and cannot be created directly.
///
/// You can access the Xot id information using the [`NameIdInfo`] trait.
///
/// You can access name string information using the [`NameStrInfo`] trait.
///
/// It can also be used directly to create new elements and attributes.
#[derive(Debug, Clone)]
pub struct Ref<'a, L: Lookup> {
    /// Looking up string information for names, namespaces and prefixes.
    xot: &'a Xot,
    // A way to look up prefix information.
    lookup: L,
    // This identifies the name and namespace. This is the only thing that
    // identifies the xml name and is used for hashing and comparison.
    name_id: NameId,
}

impl<'a, L: Lookup> std::hash::Hash for Ref<'a, L> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name_id.hash(state);
    }
}

impl<'a, L: Lookup> PartialEq for Ref<'a, L> {
    fn eq(&self, other: &Self) -> bool {
        self.name_id == other.name_id
    }
}

impl<'a, L: Lookup> Eq for Ref<'a, L> {}

#[derive(Debug, Clone)]
struct NodeLookup<'a> {
    xot: &'a Xot,
    node: Node,
}

impl<'a> Lookup for NodeLookup<'a> {
    #[inline]
    fn prefix_id_for_namespace_id(&self, namespace_id: NamespaceId) -> Option<PrefixId> {
        self.xot.prefix_for_namespace(self.node, namespace_id)
    }
}

impl<'a, L: Lookup> NameIdInfo for Ref<'a, L> {
    /// Access the underlying name id
    #[inline]
    fn name_id(&self) -> NameId {
        self.name_id
    }

    /// Access the underlying namespace id
    #[inline]
    fn namespace_id(&self) -> NamespaceId {
        self.xot.namespace_for_name(self.name_id)
    }

    /// Access the prefix id in this context.
    fn prefix_id(&self) -> Result<PrefixId, Error> {
        let namespace_id = self.namespace_id();
        self.lookup
            .prefix_id_for_namespace_id(namespace_id)
            .ok_or_else(|| Error::MissingPrefix(self.xot.namespace_str(namespace_id).to_string()))
    }
}

impl<'a, L: Lookup> NameStrInfo for Ref<'a, L> {
    #[inline]
    fn local_name(&self) -> &'a str {
        self.xot.local_name_str(self.name_id)
    }

    #[inline]
    fn namespace(&self) -> &'a str {
        self.xot.namespace_str(self.namespace_id())
    }

    #[inline]
    fn prefix(&self) -> Result<&'a str, Error> {
        let prefix_id = self.prefix_id()?;
        Ok(self.xot.prefix_str(prefix_id))
    }
}

impl<'a, L: Lookup> Ref<'a, L> {
    pub(crate) fn new(xot: &'a Xot, lookup: L, name_id: NameId) -> Self {
        Self {
            xot,
            lookup,
            name_id,
        }
    }

    /// Create a new [`State`] from this reference.
    ///
    /// This is useful if you need to store the name information in an efficient way
    /// without worrying about references.
    pub fn to_state(&self) -> Result<State, Error> {
        Ok(State::new(
            self.name_id,
            self.namespace_id(),
            self.prefix_id()?,
        ))
    }

    /// Create a new [`Owned`] from this reference.
    ///
    /// Normally you shouldn't have to do this because you can already access
    /// the name string information on this reference using [`NameStrInfo`].
    pub fn to_owned(&self) -> Result<Owned, Error> {
        Ok(Owned::new(
            self.local_name().to_string(),
            self.namespace().to_string(),
            self.prefix()?.to_string(),
        ))
    }

    /// Has a namespace but no prefix, so it's in a `xmlns` namespace.
    pub fn has_unprefixed_namespace(&self) -> Result<bool, Error> {
        Ok(self.namespace_id() != self.xot.no_namespace()
            && self.xot.empty_prefix() == self.prefix_id()?)
    }
}
