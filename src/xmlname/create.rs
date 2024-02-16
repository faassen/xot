use crate::{id::NameId, Error, Xot};

use super::owned::parse_full_name;

/// This is a convenient and efficient way to create a new name for use in Xot.
///
/// You can use it with APIs like [`Xot::new_element`] and [`xot::Attributes::insert`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct XmlNameCreate {
    name_id: NameId,
}

impl XmlNameCreate {
    pub(crate) fn new(name_id: NameId) -> Self {
        Self { name_id }
    }

    /// Create from a local name and namespace.
    ///
    /// If namespace is the empty string, the name isn't in a namespace.
    pub fn local_name_namespace(xot: &mut Xot, local_name: &str, namespace: &str) -> Self {
        let namespace_id = xot.add_namespace(namespace);
        let name_id = xot.add_name_ns(local_name, namespace_id);
        Self { name_id }
    }

    /// Create from a local name without namespace
    pub fn local_name(xot: &mut Xot, local_name: &str) -> Self {
        Self::local_name_namespace(xot, local_name, "")
    }

    /// Given prefix, and local name, create an XmlName
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

    /// Given a fullname (with potentially a prefix), construct an XmlName
    pub fn full_name(
        xot: &mut Xot,
        lookup_namespace: impl Fn(&str) -> Option<&str>,
        full_name: &str,
    ) -> Result<Self, Error> {
        let (prefix, local_name) = parse_full_name(full_name);
        Self::prefix_local_name(xot, lookup_namespace, prefix, local_name)
    }

    /// The created name id
    pub fn name_id(&self) -> NameId {
        self.name_id
    }
}

impl From<XmlNameCreate> for NameId {
    fn from(name: XmlNameCreate) -> Self {
        name.name_id
    }
}
