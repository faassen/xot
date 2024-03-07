// when we render a element or attribute name, we need to render it with the
// right prefix information. we also want to keep track of prefix declarations.
// This data structure maintains this information and avoids having to wander
// the tree to get it.

use std::borrow::Cow;

use crate::{Error, NameId, NamespaceId, PrefixId, Xot};

pub(crate) type NamespaceDeclarations = Vec<(PrefixId, NamespaceId)>;

#[derive(Debug)]
struct FullnameInfo {
    all_namespaces: NamespaceDeclarations,
}

impl FullnameInfo {
    fn new(node_namespaces: NamespaceDeclarations, current_fullname_info: &FullnameInfo) -> Self {
        let current_namespaces = current_fullname_info
            .all_namespaces
            .iter()
            // filter out any overridden prefixes
            .filter(|(p, _)| !node_namespaces.iter().any(|(p2, _)| p2 == p));

        let all_namespaces = current_namespaces
            .chain(node_namespaces.iter())
            .copied()
            .collect();
        Self { all_namespaces }
    }

    fn prefixes_by_namespace(&self, namespace: NamespaceId) -> impl Iterator<Item = PrefixId> + '_ {
        self.all_namespaces
            .iter()
            .rev()
            .filter(move |(_, n)| *n == namespace)
            .map(|(p, _)| *p)
    }

    // look for the prefix. prefer the empty prefix, and if that isn't there, the
    // most recently defined prefix
    fn element_prefix_by_namespace(&self, xot: &Xot, namespace: NamespaceId) -> Option<PrefixId> {
        if self
            .prefixes_by_namespace(namespace)
            .any(|p| p == xot.empty_prefix())
        {
            Some(xot.empty_prefix())
        } else {
            self.prefixes_by_namespace(namespace).next()
        }
    }

    // look for the prefix, but only if it's not the empty prefix, as this is
    // for attributes which cannot be unprefixed and still in a namespace
    fn attribute_prefix_by_namespace(&self, xot: &Xot, namespace: NamespaceId) -> Option<PrefixId> {
        self.prefixes_by_namespace(namespace)
            .find(|&prefix| prefix != xot.empty_prefix())
    }
}

pub(crate) struct FullnameSerializer<'a> {
    xot: &'a Xot,
    stack: Vec<FullnameInfo>,
}

impl<'a> FullnameSerializer<'a> {
    pub(crate) fn new(xot: &'a Xot, defined_namespaces: NamespaceDeclarations) -> Self {
        Self {
            xot,
            stack: vec![FullnameInfo {
                all_namespaces: defined_namespaces,
            }],
        }
    }

    pub(crate) fn push(&mut self, defined_namespaces: NamespaceDeclarations) {
        // optimization; we don't need to recalculate anything if we
        // already have the same namespaces. the cost is that we need
        // to keep track of whether this node defined namespaces for pop as well.
        if defined_namespaces.is_empty() {
            return;
        }
        let current_fullname_info = self.stack.last().unwrap();
        self.stack
            .push(FullnameInfo::new(defined_namespaces, current_fullname_info));
    }

    pub(crate) fn has_empty_prefix(&self, namespace_id: NamespaceId) -> bool {
        let prefix_id = self
            .top()
            .element_prefix_by_namespace(self.xot, namespace_id);
        if let Some(prefix_id) = prefix_id {
            prefix_id == self.xot.empty_prefix()
        } else {
            false
        }
    }

    // pub(crate) fn has_ancestor_empty_prefix(&self, namespace_id: NamespaceId) -> bool {
    //     if self.stack.len() < 2 {
    //         return false;
    //     }
    //     let prefix_id =
    //         self.stack[self.stack.len() - 2].element_prefix_by_namespace(self.xot, namespace_id);
    //     if let Some(prefix_id) = prefix_id {
    //         prefix_id == self.xot.empty_prefix()
    //     } else {
    //         false
    //     }
    // }

    // this is handy for the HTML rendering system, which insists some namespaces
    // should be in the empty prefix (xhtml, mathml, svg)
    pub(crate) fn add_empty_prefix(&mut self, namespace_id: NamespaceId) {
        let current_fullname_info = self.stack.last_mut().unwrap();
        let empty_entry = (self.xot.empty_prefix(), namespace_id);
        current_fullname_info.all_namespaces.push(empty_entry);
    }

    pub(crate) fn pop(&mut self, has_namespaces: bool) {
        if has_namespaces {
            self.stack.pop();
        }
    }

    fn top(&self) -> &FullnameInfo {
        self.stack.last().unwrap()
    }

    pub(crate) fn element_prefix(&self, name_id: NameId) -> Result<Option<PrefixId>, Error> {
        let namespace_id = self.xot.namespace_for_name(name_id);
        if namespace_id == self.xot.no_namespace_id {
            // no namespace, therefore no prefix to show
            Ok(None)
        } else {
            let prefix_id = self
                .top()
                .element_prefix_by_namespace(self.xot, namespace_id);
            if let Some(prefix_id) = prefix_id {
                if prefix_id == self.xot.empty_prefix() {
                    // empty prefix, therefore default namespace
                    Ok(None)
                } else {
                    // a prefix
                    Ok(Some(prefix_id))
                }
            } else {
                // missing prefix
                Err(Error::MissingPrefix(
                    self.xot.namespace_str(namespace_id).to_string(),
                ))
            }
        }
    }

    // get the fullname. if None, we cannot generate the fullname due to a missing
    // prefix
    pub(crate) fn element_fullname(&self, name_id: NameId) -> Result<Cow<'a, str>, Error> {
        if let Some(prefix) = self.element_prefix(name_id)? {
            Ok(Cow::Owned(format!(
                "{}:{}",
                self.xot.prefix_str(prefix),
                self.xot.local_name_str(name_id)
            )))
        } else {
            Ok(Cow::Borrowed(self.xot.local_name_str(name_id)))
        }
    }

    pub(crate) fn attribute_prefix(&self, name_id: NameId) -> Result<Option<PrefixId>, Error> {
        let namespace_id = self.xot.namespace_for_name(name_id);
        if namespace_id == self.xot.no_namespace_id {
            // no namespace, therefore no prefix to show
            Ok(None)
        } else {
            let prefix_id = self
                .top()
                .attribute_prefix_by_namespace(self.xot, namespace_id);
            if let Some(prefix_id) = prefix_id {
                // a prefix, which cannot be empty
                Ok(Some(prefix_id))
            } else {
                // missing prefix
                Err(Error::MissingPrefix(
                    self.xot.namespace_str(namespace_id).to_string(),
                ))
            }
        }
    }

    pub(crate) fn attribute_fullname(&self, name_id: NameId) -> Result<Cow<'a, str>, Error> {
        if let Some(prefix) = self.attribute_prefix(name_id)? {
            Ok(Cow::Owned(format!(
                "{}:{}",
                self.xot.prefix_str(prefix),
                self.xot.local_name_str(name_id)
            )))
        } else {
            Ok(Cow::Borrowed(self.xot.local_name_str(name_id)))
        }
    }

    pub(crate) fn is_namespace_known(&self, namespace_id: NamespaceId) -> bool {
        self.top()
            .all_namespaces
            .iter()
            .any(|(_, ns)| *ns == namespace_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_no_namespace() {
        let mut xot = Xot::new();

        let a = xot.add_name("a");
        let fullname_serializer = FullnameSerializer::new(&xot, vec![]);

        assert_eq!(
            fullname_serializer.element_fullname(a).unwrap(),
            Cow::Borrowed("a")
        );
        assert_eq!(fullname_serializer.element_prefix(a).unwrap(), None)
    }

    #[test]
    fn test_attribute_no_namespace() {
        let mut xot = Xot::new();

        let a = xot.add_name("a");
        let fullname_serializer = FullnameSerializer::new(&xot, vec![]);

        assert_eq!(
            fullname_serializer.attribute_fullname(a).unwrap(),
            Cow::Borrowed("a")
        );
        assert_eq!(fullname_serializer.element_prefix(a).unwrap(), None)
    }

    #[test]
    fn test_element_with_prefix() {
        let mut xot = Xot::new();

        let ns = xot.add_namespace("ns");
        let a = xot.add_name_ns("a", ns);
        let prefix = xot.add_prefix("p");
        let fullname_serializer = FullnameSerializer::new(&xot, vec![(prefix, ns)]);

        assert_eq!(
            fullname_serializer.element_fullname(a).unwrap(),
            Cow::Owned::<str>("p:a".to_string())
        );
        assert_eq!(fullname_serializer.element_prefix(a).unwrap(), Some(prefix))
    }

    #[test]
    fn test_attribute_with_prefix() {
        let mut xot = Xot::new();

        let ns = xot.add_namespace("ns");
        let a = xot.add_name_ns("a", ns);
        let prefix = xot.add_prefix("p");
        let fullname_serializer = FullnameSerializer::new(&xot, vec![(prefix, ns)]);

        assert_eq!(
            fullname_serializer.attribute_fullname(a).unwrap(),
            Cow::Owned::<str>("p:a".to_string())
        );
        assert_eq!(fullname_serializer.element_prefix(a).unwrap(), Some(prefix))
    }

    #[test]
    fn test_element_default_namespace() {
        let mut xot = Xot::new();

        let ns = xot.add_namespace("ns");
        let a = xot.add_name_ns("a", ns);
        let fullname_serializer = FullnameSerializer::new(&xot, vec![(xot.empty_prefix(), ns)]);

        assert_eq!(
            fullname_serializer.element_fullname(a).unwrap(),
            Cow::Borrowed("a")
        );
        assert_eq!(fullname_serializer.element_prefix(a).unwrap(), None)
    }

    #[test]
    fn test_element_default_namespace_empty_prefix_preferred() {
        let mut xot = Xot::new();

        let ns = xot.add_namespace("ns");
        let a = xot.add_name_ns("a", ns);
        let p = xot.add_prefix("p");
        let fullname_serializer =
            FullnameSerializer::new(&xot, vec![(xot.empty_prefix(), ns), (p, ns)]);

        assert_eq!(
            fullname_serializer.element_fullname(a).unwrap(),
            Cow::Borrowed("a")
        );
        assert_eq!(fullname_serializer.element_prefix(a).unwrap(), None)
    }

    #[test]
    fn test_element_most_recently_defined_prefix_preferred() {
        let mut xot = Xot::new();

        let ns = xot.add_namespace("ns");
        let a = xot.add_name_ns("a", ns);
        let p1 = xot.add_prefix("p1");
        let p2 = xot.add_prefix("p2");
        let fullname_serializer = FullnameSerializer::new(&xot, vec![(p1, ns), (p2, ns)]);

        assert_eq!(
            fullname_serializer.element_fullname(a).unwrap(),
            Cow::Owned::<str>("p2:a".to_string())
        );
        assert_eq!(fullname_serializer.element_prefix(a).unwrap(), Some(p2))
    }

    #[test]
    fn test_element_missing_prefix() {
        let mut xot = Xot::new();

        let ns = xot.add_namespace("ns");
        let a = xot.add_name_ns("a", ns);
        let fullname_serializer = FullnameSerializer::new(&xot, vec![]);

        assert!(fullname_serializer.element_fullname(a).is_err());
        assert!(fullname_serializer.element_prefix(a).is_err());
    }

    #[test]
    fn test_attribute_explicit_prefix_used_over_empty_prefix() {
        let mut xot = Xot::new();

        let ns = xot.add_namespace("ns");
        let a = xot.add_name_ns("a", ns);
        let p = xot.add_prefix("p");
        let fullname_serializer =
            FullnameSerializer::new(&xot, vec![(xot.empty_prefix(), ns), (p, ns)]);

        assert_eq!(
            fullname_serializer.attribute_fullname(a).unwrap(),
            Cow::Owned::<str>("p:a".to_string())
        );
        assert_eq!(fullname_serializer.attribute_prefix(a).unwrap(), Some(p));
    }

    #[test]
    fn test_attribute_explicit_prefix_used_over_empty_prefix2() {
        let mut xot = Xot::new();

        let ns = xot.add_namespace("ns");
        let a = xot.add_name_ns("a", ns);
        let p = xot.add_prefix("p");
        let fullname_serializer =
            FullnameSerializer::new(&xot, vec![(p, ns), (xot.empty_prefix(), ns)]);

        assert_eq!(
            fullname_serializer.attribute_fullname(a).unwrap(),
            Cow::Owned::<str>("p:a".to_string())
        );
        assert_eq!(fullname_serializer.attribute_prefix(a).unwrap(), Some(p));
    }

    #[test]
    fn test_attribute_without_explicit_prefix_default_not_found() {
        let mut xot = Xot::new();

        let ns = xot.add_namespace("ns");
        let a = xot.add_name_ns("a", ns);

        let fullname_serializer = FullnameSerializer::new(&xot, vec![(xot.empty_prefix(), ns)]);

        assert!(fullname_serializer.attribute_fullname(a).is_err());
        assert!(fullname_serializer.attribute_prefix(a).is_err());
    }

    #[test]
    fn test_overridden_prefix() {
        let mut xot = Xot::new();

        let ns1 = xot.add_namespace("ns1");
        let ns2 = xot.add_namespace("ns2");
        let a1 = xot.add_name_ns("a", ns1);
        let a2 = xot.add_name_ns("a", ns2);

        let p = xot.add_prefix("p");
        // override the prefix p so that ns1 isn't accessible anymore
        let mut fullname_serializer = FullnameSerializer::new(&xot, vec![(p, ns1)]);
        fullname_serializer.push(vec![(p, ns2)]);

        assert_eq!(
            fullname_serializer.element_fullname(a2).unwrap(),
            Cow::Owned::<str>("p:a".to_string())
        );
        assert!(fullname_serializer.attribute_fullname(a1).is_err());
        assert!(fullname_serializer.attribute_prefix(a1).is_err());
    }
}
