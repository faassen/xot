use crate::id::{NamespaceId, PrefixId};
use crate::{Error, Xot};

use super::state::State;
use super::Create;
use super::{
    reference::{Lookup, NameStrInfo},
    Ref,
};

/// An owned name stores the name, namespace and prefix as owned strings.
///
/// An owned name is handy when you don't want to depend on Xot, or for
/// debugging.
///
/// It cannot be used to create elements directly, but you can turn this into a
/// [`Create`] using [`Owned.to_create`] and a
/// [`Ref`] using [`Owned::to_ref`].
///
/// You can access name string information using the [`NameStrInfo`] trait.
///
/// If you enable the `serde` feature it can be serialized and deserialized.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Owned {
    local_name: String,
    // the empty namespace uri means no namespace
    namespace: String,
    // the empty prefix means no prefix.
    prefix: String,
}

impl std::hash::Hash for Owned {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.local_name.hash(state);
        self.namespace.hash(state);
    }
}

impl PartialEq for Owned {
    fn eq(&self, other: &Self) -> bool {
        self.local_name == other.local_name && self.namespace == other.namespace
    }
}

impl Eq for Owned {}

impl NameStrInfo for Owned {
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

impl Owned {
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

    /// create a new owned name from a prefix and a name.
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
        lookup_namespace: impl Fn(&str) -> Option<String>,
    ) -> Result<Self, Error> {
        let (prefix, local_name) = parse_full_name(full_name);
        let namespace =
            lookup_namespace(prefix).ok_or_else(|| Error::UnknownPrefix(prefix.to_string()))?;
        Ok(Self {
            local_name: local_name.to_string(),
            namespace: namespace.to_string(),
            prefix: prefix.to_string(),
        })
    }

    /// Create a new [`Ref`] from this owned.
    pub fn to_ref<'a>(&self, xot: &'a mut Xot) -> Ref<'a, PrefixIdLookup> {
        let prefix_id = xot.add_prefix(&self.prefix);
        let lookup = PrefixIdLookup { prefix_id };
        let namespace_id = xot.add_namespace(&self.namespace);
        let name_id = xot.add_name_ns(&self.local_name, namespace_id);
        Ref::new(xot, lookup, name_id)
    }

    /// Create a new [`Create`] from this owned.
    ///
    /// This disregards the prefix information.
    pub fn to_create(&self, xot: &mut Xot) -> Create {
        let namespace_id = xot.add_namespace(&self.namespace);
        let name_id = xot.add_name_ns(&self.local_name, namespace_id);
        Create::new(name_id)
    }
    /// Create a new [`State`] from this owned.
    pub fn to_state(&self, xot: &mut Xot) -> Result<State, Error> {
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
