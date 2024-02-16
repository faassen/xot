use crate::id::{NamespaceId, PrefixId};
use crate::{Error, Xot};

use super::state::XmlNameState;
use super::{
    reference::{Lookup, NameStrInfo},
    XmlNameRef,
};

/// An owned name stores the name, namespace and prefix as owned strings.
///
/// An owned name is handy when you don't want to depend on Xot, or for
/// debugging.
///
/// You can turn this into a [`XmlNameRef`] using [`XmlNameOwned::to_ref`].
/// This allows you to access both id and string information.
///
/// You can access name string information using the [`NameStrInfo`] trait.
///
/// If you enable the `serde` feature it can be serialized and deserialized.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XmlNameOwned {
    local_name: String,
    // the empty namespace uri means no namespace
    namespace: String,
    // the empty prefix means no prefix
    prefix: String,
}

impl std::hash::Hash for XmlNameOwned {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.local_name.hash(state);
        self.namespace.hash(state);
    }
}

impl PartialEq for XmlNameOwned {
    fn eq(&self, other: &Self) -> bool {
        self.local_name == other.local_name && self.namespace == other.namespace
    }
}

impl Eq for XmlNameOwned {}

impl NameStrInfo for XmlNameOwned {
    fn local_name(&self) -> &str {
        &self.local_name
    }

    fn namespace(&self) -> &str {
        &self.namespace
    }

    fn prefix(&self) -> Result<&str, Error> {
        Ok(&self.prefix)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PrefixIdLookup {
    prefix_id: PrefixId,
}

impl Lookup for PrefixIdLookup {
    fn prefix_id_for_namespace_id(&self, _namespace_id: NamespaceId) -> Option<PrefixId> {
        Some(self.prefix_id)
    }
    fn namespace_id_for_prefix_id(&self, _prefix_id: PrefixId) -> Option<NamespaceId> {
        // we already know the namespace id in this case, so there is no need
        // to look it up
        unreachable!()
    }
}

impl XmlNameOwned {
    /// Create a new owned XmlName
    pub fn new(local_name: String, namespace: String, prefix: String) -> Self {
        Self {
            local_name,
            namespace,
            prefix,
        }
    }

    /// Create a new owned XmlName without prefix information.
    pub fn new_no_prefix(local_name: String, namespace: String) -> Self {
        Self {
            local_name,
            namespace,
            prefix: String::new(),
        }
    }

    /// Create a new owned XmlName while looking up the prefix information.
    pub fn new_lookup_prefix(
        prefix_lookup: impl Fn(&str) -> Option<String>,
        local_name: String,
        namespace: String,
    ) -> Result<Self, Error> {
        let prefix =
            prefix_lookup(&namespace).ok_or_else(|| Error::MissingPrefix(namespace.clone()))?;
        Ok(Self {
            local_name,
            namespace,
            prefix,
        })
    }

    /// Given a fullname (with potentially a prefix), construct an XmlNameOwned
    ///
    /// This requires a function that can look up the namespace for a prefix.
    pub fn from_full_name<L: Lookup>(
        namespace_lookup: impl Fn(&str) -> Option<String>,
        full_name: &str,
    ) -> Result<Self, Error> {
        let (prefix, local_name) = parse_full_name(full_name);
        let namespace =
            namespace_lookup(prefix).ok_or_else(|| Error::UnknownPrefix(prefix.to_string()))?;
        Ok(Self {
            local_name: local_name.to_string(),
            namespace: namespace.to_string(),
            prefix: prefix.to_string(),
        })
    }

    /// Create a new [`crate::xmlname::XmlNameRef`] from this owned.
    pub fn to_ref<'a>(&self, xot: &'a mut Xot) -> XmlNameRef<'a, PrefixIdLookup> {
        let prefix_id = xot.add_prefix(&self.prefix);
        let lookup = PrefixIdLookup { prefix_id };

        XmlNameRef::from_local_name_namespace(xot, lookup, &self.local_name, &self.namespace)
    }

    /// Create a new [`crate::xmlname::XmlNameState`] from this owned.
    pub fn to_state(&self, xot: &mut Xot) -> Result<XmlNameState, Error> {
        self.to_ref(xot).to_state()
    }
}

/// TODO: fancier parser that validates the name
pub(crate) fn parse_full_name(full_name: &str) -> (&str, &str) {
    match full_name.find(':') {
        Some(pos) => {
            let (prefix, local_name) = full_name.split_at(pos);
            (prefix, &local_name[1..])
        }
        None => ("", full_name),
    }
}
