use crate::{NameId, PrefixId, Xot};

trait QName {
    /// The local part of the name.
    fn local(&self) -> &str;

    /// The namespace URI.
    /// If None, this name is not in a namespace.
    fn uri(&self) -> Option<&str>;

    /// The prefix of the name.
    ///
    /// This prefix may be the empty string. The prefix information
    /// may also be missing, in which case this is None.
    fn prefix(&self) -> Option<&str>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StringBackedQName {
    local: String,
    uri: Option<String>,
    prefix: Option<String>,
}

impl StringBackedQName {
    pub fn new(local: String, uri: Option<String>, prefix: Option<String>) -> Self {
        Self { local, uri, prefix }
    }

    pub fn without_namespace(local: String) -> Self {
        Self::new(local, None, None)
    }

    pub fn without_prefix(local: String, uri: String) -> Self {
        Self::new(local, Some(uri), None)
    }

    pub fn prefixed<'a>(
        prefix: &'a str,
        local: String,
        namespaces: impl QNamespaces<'a>,
    ) -> Option<Self> {
        let uri = namespaces.by_prefix(prefix)?;
        Some(Self::new(
            local,
            Some(uri.to_string()),
            Some(prefix.to_string()),
        ))
    }
}

impl QName for StringBackedQName {
    fn local(&self) -> &str {
        &self.local
    }

    fn uri(&self) -> Option<&str> {
        self.uri.as_deref()
    }

    fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }
}

#[derive(Clone)]
struct XotBackedQName<'a> {
    xot: &'a Xot,
    name_id: NameId,
    prefix_id: Option<PrefixId>,
}

impl<'a> XotBackedQName<'a> {
    pub fn new(
        local: String,
        uri: Option<String>,
        prefix: Option<String>,
        xot: &'a mut Xot,
    ) -> Self {
        let name_id = if let Some(uri) = uri {
            let namespace_id = xot.add_namespace(&uri);
            xot.add_name_ns(&local, namespace_id)
        } else {
            xot.add_name(&local)
        };
        let prefix_id = prefix.map(|prefix| xot.add_prefix(&prefix));
        Self {
            xot,
            name_id,
            prefix_id,
        }
    }

    pub fn without_namespace(local: String, xot: &'a mut Xot) -> Self {
        Self::new(local, None, None, xot)
    }

    pub fn without_prefix(local: String, uri: String, xot: &'a mut Xot) -> Self {
        Self::new(local, Some(uri), None, xot)
    }

    pub fn prefixed<'n>(
        prefix: &'n str,
        local: String,
        namespaces: impl QNamespaces<'n>,
        xot: &'a mut Xot,
    ) -> Option<Self> {
        let uri = namespaces.by_prefix(prefix)?;
        Some(Self::new(
            local,
            Some(uri.to_string()),
            Some(prefix.to_string()),
            xot,
        ))
    }
}

impl<'a> QName for XotBackedQName<'a> {
    fn local(&self) -> &str {
        self.xot.name_ns_str(self.name_id).0
    }

    fn uri(&self) -> Option<&str> {
        let namespace_id = self.xot.namespace_for_name(self.name_id);
        if namespace_id != self.xot.no_namespace() {
            Some(self.xot.namespace_str(namespace_id))
        } else {
            None
        }
    }

    fn prefix(&self) -> Option<&str> {
        self.prefix_id
            .map(|prefix_id| self.xot.prefix_str(prefix_id))
    }
}

trait QNamespaces<'a> {
    fn by_prefix(&self, prefix: &'a str) -> Option<&'a str>;

    // fn prefixes(&self) -> PrefixIterator<'a>;
}

// struct PrefixIterator<'a> {}

// impl Iterator for PrefixIterator<'a> {
//     type Item = String;
//     fn next(&mut self) -> Option<Self::Item> {
//         None
//     }
// }

const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";
