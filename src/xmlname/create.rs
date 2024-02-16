use crate::{id::NameId, Error, Xot};

use super::owned::parse_full_name;

/// This is a convenient and efficient way to create a new name for use in Xot.
///
/// You can use it with APIs like [`Xot::new_element`] and [`crate::MutableAttributes::insert`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Create {
    name_id: NameId,
}

impl Create {
    #[inline]
    pub(crate) fn new(name_id: NameId) -> Self {
        Self { name_id }
    }

    /// A name in a namespace.
    ///
    /// If namespace is the empty string, the name isn't in a namespace.
    #[inline]
    pub fn local_name_namespace(xot: &mut Xot, local_name: &str, namespace: &str) -> Self {
        let namespace_id = xot.add_namespace(namespace);
        let name_id = xot.add_name_ns(local_name, namespace_id);
        Self { name_id }
    }

    /// A name without a namespace.
    #[inline]
    pub fn local_name(xot: &mut Xot, local_name: &str) -> Self {
        Self::local_name_namespace(xot, local_name, "")
    }

    /// A name given a prefix. The prefix is looked up in the provided function.
    pub fn prefix_local_name(
        xot: &mut Xot,
        lookup_namespace: impl Fn(&str) -> Option<&str>,
        prefix: &str,
        local_name: &str,
    ) -> Result<Self, Error> {
        let namespace =
            lookup_namespace(prefix).ok_or_else(|| Error::UnknownPrefix(prefix.to_string()))?;
        Ok(Self::local_name_namespace(xot, local_name, namespace))
    }

    /// Parse a fullname (with potentially a prefix) and construct a name.
    ///
    /// The prefix is looked up in the provided function.
    pub fn parse_full_name(
        xot: &mut Xot,
        lookup_namespace: impl Fn(&str) -> Option<&str>,
        full_name: &str,
    ) -> Result<Self, Error> {
        let (prefix, local_name) = parse_full_name(full_name);
        Self::prefix_local_name(xot, lookup_namespace, prefix, local_name)
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
