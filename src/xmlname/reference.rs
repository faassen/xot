use std::borrow::Cow;

use crate::id::{NameId, NamespaceId, PrefixId};
use crate::xotdata::Xot;
use crate::{Error, Node};

use super::owned::OwnedName;

/// Name string information for an xml name.
pub trait NameStrInfo {
    /// Access the local name as a string reference
    fn local_name(&self) -> &str;

    /// Get the namespace uri as a str reference.
    ///
    /// If there is no namespace, this is the empty string.
    fn namespace(&self) -> &str;

    /// Access the prefix as a string
    fn prefix(&self) -> &str;

    /// Access the full name as a string
    fn full_name(&self) -> Cow<str> {
        let prefix = self.prefix();
        if !prefix.is_empty() {
            Cow::Owned(format!("{}:{}", prefix, self.local_name()))
        } else {
            Cow::Borrowed(self.local_name())
        }
    }
}

/// The most complete way to access name information, backed by Xot. This is a
/// reference and cannot be created directly.
///
/// You can create one by using [`OwnedName`], or by using the
/// [`Xot::name_ref`] and [`Xot::node_name_ref`] methods.
///
/// You can access name string information using the [`NameStrInfo`] trait.
///
/// It can also be used directly to create new elements and attributes, instead
/// of a [`crate::NameId`].
#[derive(Debug, Clone)]
pub struct RefName<'a> {
    /// Looking up string information for names, namespaces and prefixes.
    xot: &'a Xot,
    // This identifies the name and namespace. This is the only thing that
    // identifies the xml name and is used for hashing and comparison.
    name_id: NameId,
    /// We also keep track of the prefix id
    prefix_id: PrefixId,
}

impl std::hash::Hash for RefName<'_> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name_id.hash(state);
    }
}

impl PartialEq for RefName<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.name_id == other.name_id
    }
}

impl Eq for RefName<'_> {}

impl<'a> NameStrInfo for RefName<'a> {
    #[inline]
    fn local_name(&self) -> &'a str {
        self.xot.local_name_str(self.name_id)
    }

    #[inline]
    fn namespace(&self) -> &'a str {
        self.xot.namespace_str(self.namespace_id())
    }

    #[inline]
    fn prefix(&self) -> &'a str {
        let prefix_id = self.prefix_id();
        self.xot.prefix_str(prefix_id)
    }
}

impl<'a> RefName<'a> {
    pub(crate) fn new(xot: &'a Xot, name_id: NameId, prefix_id: PrefixId) -> Self {
        Self {
            xot,
            name_id,
            prefix_id,
        }
    }

    /// Get the Xot name id.
    #[inline]
    pub fn name_id(&self) -> NameId {
        self.name_id
    }

    /// Get the Xot namespace id.
    #[inline]
    pub fn namespace_id(&self) -> NamespaceId {
        self.xot.namespace_for_name(self.name_id)
    }

    /// Get the Xot prefix id.
    #[inline]
    pub fn prefix_id(&self) -> PrefixId {
        self.prefix_id
    }

    pub(crate) fn from_node(xot: &'a Xot, node: Node, name_id: NameId) -> Result<Self, Error> {
        let namespace_id = xot.namespace_for_name(name_id);
        let prefix_id = if namespace_id != xot.no_namespace() {
            xot.prefix_for_namespace(node, namespace_id)
                .ok_or_else(|| Error::MissingPrefix(xot.namespace_str(namespace_id).to_string()))?
        } else {
            xot.empty_prefix()
        };
        Ok(Self::new(xot, name_id, prefix_id))
    }

    /// Create a new [`OwnedName`] from this reference.
    ///
    /// Normally you shouldn't have to do this because you can already access
    /// the name string information on this reference using [`NameStrInfo`].
    pub fn to_owned(&self) -> OwnedName {
        OwnedName::new(
            self.local_name().to_string(),
            self.namespace().to_string(),
            self.prefix().to_string(),
        )
    }

    /// Has a namespace but no prefix, so it's in a `xmlns` namespace.
    pub fn has_unprefixed_namespace(&self) -> bool {
        self.namespace_id() != self.xot.no_namespace()
            && self.xot.empty_prefix() == self.prefix_id()
    }
}

impl<'a> From<RefName<'a>> for NameId {
    #[inline]
    fn from(name: RefName<'a>) -> Self {
        name.name_id
    }
}
