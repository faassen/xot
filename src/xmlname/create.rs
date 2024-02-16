use crate::{id::NameId, Error, NamespaceId, PrefixId, Xot};

use super::owned::parse_full_name;

/// This is a convenient and efficient way to create a new name for use in Xot.
///
/// You can use it with APIs like [`Xot::new_element`] and
/// [`crate::MutableAttributes::insert`].
///
/// Prefix information is not maintained; this can be derived from context
/// after it's used or converted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Create {
    name_id: NameId,
}

impl Create {
    #[inline]
    pub(crate) fn new(name_id: NameId) -> Self {
        Self { name_id }
    }

    /// Create a name without a namespace.
    #[inline]
    pub fn name(xot: &mut Xot, local_name: &str) -> Self {
        let name_id = xot.add_name(local_name);
        Self { name_id }
    }

    /// Create a name in a namespace.
    ///
    /// If namespace is the empty string, the name isn't in a namespace.
    #[inline]
    pub fn namespaced(xot: &mut Xot, local_name: &str, namespace: &Namespace) -> Self {
        let name_id = xot.add_name_ns(local_name, namespace.namespace_id());
        Self { name_id }
    }

    /// A name given a prefix.
    ///
    /// The namespace is looked up in the provided function.
    pub fn prefixed(
        xot: &mut Xot,
        prefix: &str,
        local_name: &str,
        lookup_namespace: impl Fn(&str) -> Option<NamespaceId>,
    ) -> Result<Self, Error> {
        let namespace =
            lookup_namespace(prefix).ok_or_else(|| Error::UnknownPrefix(prefix.to_string()))?;
        let name_id = xot.add_name_ns(local_name, namespace);
        Ok(Self { name_id })
    }

    /// Parse a fullname (with potentially a prefix) and construct a name.
    ///
    /// The namespace is looked up in the provided function.
    pub fn parse_full_name(
        xot: &mut Xot,
        full_name: &str,
        lookup_namespace: impl Fn(&str) -> Option<NamespaceId>,
    ) -> Result<Self, Error> {
        let (prefix, local_name) = parse_full_name(full_name);
        Self::prefixed(xot, prefix, local_name, lookup_namespace)
    }

    /// The created name id.
    ///
    /// Note that you can also use `create.into()` to convert to a `NameId`.
    #[inline]
    pub fn name_id(&self) -> NameId {
        self.name_id
    }
}

impl From<Create> for NameId {
    #[inline]
    fn from(name: Create) -> Self {
        name.name_id
    }
}

/// A convenient way to create a namespace prefix to namespace URI mapping.
///
/// This can be used with [`Xot::append_namespace`] to add this mapping to the
/// tree.
///
/// You can pass it as an argument into `Create` to create an object in a namespace.
pub struct Namespace {
    pub(crate) prefix_id: PrefixId,
    pub(crate) namespace_id: NamespaceId,
}

impl Namespace {
    /// Create a new namespace prefix.
    #[inline]
    pub fn new(xot: &mut Xot, prefix: &str, namespace: &str) -> Self {
        let prefix_id = xot.add_prefix(prefix);
        let namespace_id = xot.add_namespace(namespace);
        Namespace {
            prefix_id,
            namespace_id,
        }
    }

    /// Get the prefix id
    #[inline]
    pub fn prefix_id(&self) -> PrefixId {
        self.prefix_id
    }

    /// Get the namespace id
    #[inline]
    pub fn namespace_id(&self) -> NamespaceId {
        self.namespace_id
    }
}
