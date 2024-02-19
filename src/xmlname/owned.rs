use crate::{Error, Xot};

use super::state::StateName;
use super::CreateName;
use super::{reference::NameStrInfo, RefName};

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
    // fields are postfixed _str so they don't conflict with the methods in
    // NameStrInfo
    local_name_str: String,
    // the empty namespace uri means no namespace
    namespace_str: String,
    // the empty prefix means no prefix.
    prefix_str: String,
}

impl std::hash::Hash for OwnedName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.local_name_str.hash(state);
        self.namespace_str.hash(state);
    }
}

impl PartialEq for OwnedName {
    fn eq(&self, other: &Self) -> bool {
        self.local_name_str == other.local_name_str && self.namespace_str == other.namespace_str
    }
}

impl Eq for OwnedName {}

impl NameStrInfo for OwnedName {
    fn local_name(&self) -> &str {
        &self.local_name_str
    }

    fn namespace(&self) -> &str {
        &self.namespace_str
    }

    fn prefix(&self) -> &str {
        &self.prefix_str
    }
}

impl OwnedName {
    /// Create a new owned name.
    pub fn new(local_name: String, namespace: String, prefix: String) -> Self {
        Self {
            local_name_str: local_name,
            namespace_str: namespace,
            prefix_str: prefix,
        }
    }

    /// Create a name without a namespace
    pub fn name(local_name: &str) -> Self {
        Self {
            local_name_str: local_name.to_string(),
            namespace_str: String::new(),
            prefix_str: String::new(),
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
            local_name_str: local_name,
            namespace_str: namespace,
            prefix_str: prefix,
        })
    }

    /// Create a new owned name from a prefix and a name.
    pub fn prefixed(
        prefix: &str,
        local_name: &str,
        lookup_namespace: impl Fn(&str) -> Option<String>,
    ) -> Result<Self, Error> {
        let namespace =
            lookup_namespace(prefix).ok_or_else(|| Error::UnknownPrefix(prefix.to_string()))?;
        Ok(Self {
            local_name_str: local_name.to_string(),
            namespace_str: namespace,
            prefix_str: prefix.to_string(),
        })
    }

    /// Given a fullname (with potentially a prefix), construct an XmlNameOwned
    ///
    /// This requires a function that can look up the namespace for a prefix.
    pub fn parse_full_name(
        full_name: &str,
        lookup_namespace: impl Fn(&str) -> Option<String>,
    ) -> Result<Self, Error> {
        let (prefix, local_name) = parse_full_name(full_name);
        Self::prefixed(prefix, local_name, lookup_namespace)
    }

    /// Convert this name into a name adding a * suffix.
    ///
    /// This can be useful to help generate unique names.
    pub fn with_suffix(self) -> Self {
        let mut local_name = self.local_name_str;
        local_name.push('*');
        Self {
            local_name_str: local_name,
            namespace_str: self.namespace_str,
            prefix_str: self.prefix_str,
        }
    }

    /// Convert this name into a name that's in a particular namespace.
    ///
    /// This only changes the namespace if there is an empty prefix and the
    /// namespace is not set (the empty string).
    pub fn with_default_namespace(self, namespace: &str) -> Self {
        if !self.prefix_str.is_empty() && self.namespace_str.is_empty() {
            return self;
        }
        Self {
            local_name_str: self.local_name_str,
            namespace_str: namespace.to_string(),
            prefix_str: self.prefix_str,
        }
    }

    /// Name is in a namespace but without a prefix, so it's in a default namespace.
    pub fn in_default_namespace(&self) -> bool {
        !self.namespace_str.is_empty() && self.prefix_str.is_empty()
    }

    /// Create a new [`RefName`] from this owned name.
    pub fn to_ref<'a>(&self, xot: &'a mut Xot) -> RefName<'a> {
        let prefix_id = xot.add_prefix(&self.prefix_str);
        let namespace_id = xot.add_namespace(&self.namespace_str);
        let name_id = xot.add_name_ns(&self.local_name_str, namespace_id);
        RefName::new(xot, name_id, prefix_id)
    }

    /// Create a new [`RefName`] only if the names already exists in Xot.
    ///
    /// Ignores prefix information - if the prefix doesn't exist, we still get
    /// a ref name, just without a prefix. This is because the prefix
    /// information is irrelevant for comparison.
    ///
    /// Otherwise, returns None.
    pub fn maybe_to_ref<'a>(&self, xot: &'a Xot) -> Option<RefName<'a>> {
        let prefix_id = xot.prefix(&self.prefix_str).unwrap_or(xot.empty_prefix());
        let namespace_id = xot.namespace(&self.namespace_str)?;
        let name_id = xot.name_ns(&self.local_name_str, namespace_id)?;
        Some(RefName::new(xot, name_id, prefix_id))
    }

    /// Create a new [`CreateName`] name from this owned name.
    ///
    /// This disregards the prefix information.
    pub fn to_create(&self, xot: &mut Xot) -> CreateName {
        let namespace_id = xot.add_namespace(&self.namespace_str);
        let name_id = xot.add_name_ns(&self.local_name_str, namespace_id);
        CreateName::new(name_id)
    }
    /// Create a new [`StateName`] from this owned name.
    pub fn to_state(&self, xot: &mut Xot) -> StateName {
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
