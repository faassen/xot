use crate::id::{NamespaceId, PrefixId};
use crate::{Error, Xot};

use super::state::StateName;
use super::CreateName;
use super::{
    reference::{Lookup, NameStrInfo},
    RefName,
};

/// An owned name stores the name, namespace and prefix as owned strings.
///
/// An owned name is handy when you don't want to depend on Xot, or for
/// debugging.
///
/// It cannot be used to create elements directly, but you can turn this into a
/// [`CreateName`] using [`OwnedName::to_create`] and a [`RefName`] using
/// [`OwnedName::to_ref`].
///
/// You can access name string information using the [`NameStrInfo`] trait.
///
/// If you enable the `serde` feature it can be serialized and deserialized.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OwnedName {
    local_name: String,
    // the empty namespace uri means no namespace
    namespace: String,
    // the empty prefix means no prefix.
    prefix: String,
}

impl std::hash::Hash for OwnedName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.local_name.hash(state);
        self.namespace.hash(state);
    }
}

impl PartialEq for OwnedName {
    fn eq(&self, other: &Self) -> bool {
        self.local_name == other.local_name && self.namespace == other.namespace
    }
}

impl Eq for OwnedName {}

impl NameStrInfo for OwnedName {
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
}

impl OwnedName {
    /// Create a new owned name.
    pub fn new(local_name: String, namespace: String, prefix: String) -> Self {
        Self {
            local_name,
            namespace,
            prefix,
        }
    }

    /// Create a name without a namespace
    pub fn name(local_name: &str) -> Self {
        Self {
            local_name: local_name.to_string(),
            namespace: String::new(),
            prefix: String::new(),
        }
    }

    // TODO: do we need a way to create a name without prefix information?
    // /// Create a name in a namespace, without prefix information.
    // pub fn namespaced(local_name: &str, uri: &str) -> Self {
    //     Self {
    //         local_name: local_name.to_string(),
    //         namespace: uri.to_string(),
    //         prefix: String::new(),
    //     }
    // }

    /// Create a new name in a namespace, look up prefix information.
    pub fn namespaced(
        local_name: String,
        namespace: String,
        prefix_lookup: impl Fn(&str) -> Option<String>,
    ) -> Result<Self, Error> {
        let prefix =
            prefix_lookup(&namespace).ok_or_else(|| Error::MissingPrefix(namespace.clone()))?;
        Ok(Self {
            local_name,
            namespace,
            prefix,
        })
    }

    /// Create a new owned name from a prefix and a name.
    pub fn prefixed(
        prefix: &str,
        local_name: &str,
        lookup_namespace: impl Fn(&str) -> Option<&str>,
    ) -> Result<Self, Error> {
        let namespace =
            lookup_namespace(prefix).ok_or_else(|| Error::UnknownPrefix(prefix.to_string()))?;
        Ok(Self {
            local_name: local_name.to_string(),
            namespace: namespace.to_string(),
            prefix: prefix.to_string(),
        })
    }

    /// Given a fullname (with potentially a prefix), construct an XmlNameOwned
    ///
    /// This requires a function that can look up the namespace for a prefix.
    pub fn parse_full_name<L: Lookup>(
        full_name: &str,
        lookup_namespace: impl Fn(&str) -> Option<&str>,
    ) -> Result<Self, Error> {
        let (prefix, local_name) = parse_full_name(full_name);
        Self::prefixed(prefix, local_name, lookup_namespace)
    }

    /// Convert this name into a name adding a * suffix.
    ///
    /// This can be useful to help generate unique names.
    pub fn with_suffix(self) -> Self {
        let mut local_name = self.local_name;
        local_name.push('*');
        Self {
            local_name,
            namespace: self.namespace,
            prefix: self.prefix,
        }
    }

    /// Create a new [`RefName`] from this owned name.
    pub fn to_ref<'a>(&self, xot: &'a mut Xot) -> RefName<'a, PrefixIdLookup> {
        let prefix_id = xot.add_prefix(&self.prefix);
        let lookup = PrefixIdLookup { prefix_id };
        let namespace_id = xot.add_namespace(&self.namespace);
        let name_id = xot.add_name_ns(&self.local_name, namespace_id);
        RefName::new(xot, lookup, name_id)
    }

    /// Create a new [`CreateName`] name from this owned name.
    ///
    /// This disregards the prefix information.
    pub fn to_create(&self, xot: &mut Xot) -> CreateName {
        let namespace_id = xot.add_namespace(&self.namespace);
        let name_id = xot.add_name_ns(&self.local_name, namespace_id);
        CreateName::new(name_id)
    }
    /// Create a new [`StateName`] from this owned name.
    pub fn to_state(&self, xot: &mut Xot) -> Result<StateName, Error> {
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
