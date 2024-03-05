// when we render a element or attribute name, we need to render it with the
// right prefix information. we also want to keep track of prefix declarations.
// This data structure maintains this information and avoids having to wander
// the tree to get it.

use std::borrow::Cow;

use crate::{NameId, NamespaceId, PrefixId, Xot};

type Namespaces = Vec<(PrefixId, NamespaceId)>;

// this structure maintains everything we need to know about a name per element
#[derive(Debug)]
struct FullnameInfo {
    // all namespaces defined on this node
    defined_namespaces: Namespaces,
    // namespaces known on this node
    all_namespaces: Namespaces,
}

impl FullnameInfo {
    fn new(node_namespaces: Namespaces, current_fullname_info: &FullnameInfo) -> Self {
        let all_namespaces = current_fullname_info
            .all_namespaces
            .iter()
            .chain(node_namespaces.iter())
            .copied()
            .collect();
        Self {
            defined_namespaces: node_namespaces,
            all_namespaces,
        }
    }

    fn namespace_by_prefix(&self, prefix: PrefixId) -> Option<NamespaceId> {
        self.all_namespaces
            .iter()
            .rev()
            .find(|(p, _)| *p == prefix)
            .map(|(_, ns)| *ns)
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
    pub(crate) fn new(xot: &'a Xot, defined_namespaces: Namespaces) -> Self {
        Self {
            xot,
            stack: vec![FullnameInfo {
                defined_namespaces: defined_namespaces.clone(),
                all_namespaces: defined_namespaces,
            }],
        }
    }

    pub(crate) fn push(&mut self, defined_namespaces: Namespaces) {
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

    // this is handy for the HTML rendering system, which insists some namespaces
    // should be in the empty prefix (xhtml, mathml, svg)
    pub(crate) fn add_empty_prefix(&mut self, namespace_id: NamespaceId) {
        let current_fullname_info = self.stack.last_mut().unwrap();
        let empty_entry = (self.xot.empty_prefix(), namespace_id);
        current_fullname_info.defined_namespaces.push(empty_entry);
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

    // get the fullname. if None, we cannot generate the fullname due to a missing
    // prefix
    pub(crate) fn fullname_element(&self, name_id: NameId) -> Option<Cow<'a, str>> {
        let namespace_id = self.xot.namespace_for_name(name_id);
        if namespace_id == self.xot.no_namespace_id {
            Some(Cow::Borrowed(self.xot.local_name_str(name_id)))
        } else {
            let prefix_id = self
                .top()
                .element_prefix_by_namespace(self.xot, namespace_id)?;
            if prefix_id == self.xot.empty_prefix() {
                Some(Cow::Borrowed(self.xot.local_name_str(name_id)))
            } else {
                let prefix = self.xot.prefix_str(prefix_id);
                Some(Cow::Owned(format!(
                    "{}:{}",
                    prefix,
                    self.xot.local_name_str(name_id)
                )))
            }
        }
    }

    pub(crate) fn fullname_attribute(&self, name_id: NameId) -> Option<Cow<'a, str>> {
        let namespace_id = self.xot.namespace_for_name(name_id);
        if namespace_id == self.xot.no_namespace_id {
            Some(Cow::Borrowed(self.xot.local_name_str(name_id)))
        } else {
            let prefix_id = self
                .top()
                .attribute_prefix_by_namespace(self.xot, namespace_id)?;
            let prefix = self.xot.prefix_str(prefix_id);
            Some(Cow::Owned(format!(
                "{}:{}",
                prefix,
                self.xot.local_name_str(name_id)
            )))
        }
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
            fullname_serializer.fullname_element(a),
            Some(Cow::Borrowed("a"))
        );
    }

    #[test]
    fn test_attribute_no_namespace() {
        let mut xot = Xot::new();

        let a = xot.add_name("a");
        let fullname_serializer = FullnameSerializer::new(&xot, vec![]);

        assert_eq!(
            fullname_serializer.fullname_attribute(a),
            Some(Cow::Borrowed("a"))
        );
    }

    #[test]
    fn test_element_with_prefix() {
        let mut xot = Xot::new();

        let ns = xot.add_namespace("ns");
        let a = xot.add_name_ns("a", ns);
        let prefix = xot.add_prefix("p");
        let fullname_serializer = FullnameSerializer::new(&xot, vec![(prefix, ns)]);

        assert_eq!(
            fullname_serializer.fullname_element(a),
            Some(Cow::Owned("p:a".to_string()))
        );
    }

    #[test]
    fn test_attribute_with_prefix() {
        let mut xot = Xot::new();

        let ns = xot.add_namespace("ns");
        let a = xot.add_name_ns("a", ns);
        let prefix = xot.add_prefix("p");
        let fullname_serializer = FullnameSerializer::new(&xot, vec![(prefix, ns)]);

        assert_eq!(
            fullname_serializer.fullname_attribute(a),
            Some(Cow::Owned("p:a".to_string()))
        );
    }

    #[test]
    fn test_element_default_namespace() {
        let mut xot = Xot::new();

        let ns = xot.add_namespace("ns");
        let a = xot.add_name_ns("a", ns);
        let fullname_serializer = FullnameSerializer::new(&xot, vec![(xot.empty_prefix(), ns)]);

        assert_eq!(
            fullname_serializer.fullname_element(a),
            Some(Cow::Borrowed("a"))
        );
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
            fullname_serializer.fullname_element(a),
            Some(Cow::Borrowed("a"))
        );
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
            fullname_serializer.fullname_element(a),
            Some(Cow::Owned("p2:a".to_string()))
        );
    }

    #[test]
    fn test_element_missing_prefix() {
        let mut xot = Xot::new();

        let ns = xot.add_namespace("ns");
        let a = xot.add_name_ns("a", ns);
        let fullname_serializer = FullnameSerializer::new(&xot, vec![]);

        assert_eq!(fullname_serializer.fullname_element(a), None);
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
            fullname_serializer.fullname_attribute(a),
            Some(Cow::Owned("p:a".to_string()))
        );
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
            fullname_serializer.fullname_attribute(a),
            Some(Cow::Owned("p:a".to_string()))
        );
    }

    #[test]
    fn test_attribute_without_explicit_prefix_default_not_found() {
        let mut xot = Xot::new();

        let ns = xot.add_namespace("ns");
        let a = xot.add_name_ns("a", ns);

        let fullname_serializer = FullnameSerializer::new(&xot, vec![(xot.empty_prefix(), ns)]);

        assert_eq!(fullname_serializer.fullname_attribute(a), None);
    }
}
