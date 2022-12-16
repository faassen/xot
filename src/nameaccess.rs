use ahash::{HashMap, HashSet};

use crate::access::NodeEdge;
use crate::error::Error;
use crate::name::{Name, NameId};
use crate::namespace::NamespaceId;
use crate::prefix::PrefixId;
use crate::serialize::{Fullname, FullnameSerializer};
use crate::xmlvalue::ToNamespace;
use crate::xotdata::{Node, Xot};

/// ## Names, namespaces and prefixes.
impl Xot {
    /// Look up name without a namespace.
    pub fn name(&self, name: &str) -> Option<NameId> {
        self.name_ns(name, self.no_namespace_id)
    }

    /// Add name without a namespace.
    ///
    /// If the name already exists, return its id.
    pub fn add_name(&mut self, name: &str) -> NameId {
        self.add_name_ns(name, self.no_namespace_id)
    }

    /// Look up name with a namespace.
    pub fn name_ns(&self, name: &str, namespace_id: NamespaceId) -> Option<NameId> {
        self.name_lookup
            .get_id(&Name::new(name.to_string(), namespace_id))
    }

    /// Add name with a namespace.
    ///
    /// If the name already exists, return its id.
    pub fn add_name_ns(&mut self, name: &str, namespace_id: NamespaceId) -> NameId {
        self.name_lookup
            .get_id_mut(&Name::new(name.to_string(), namespace_id))
    }

    /// Look up namespace.
    pub fn namespace(&self, namespace: &str) -> Option<NamespaceId> {
        self.namespace_lookup.get_id(namespace)
    }

    /// Add namespace.
    ///
    /// If the namespace already exists, return its id.
    pub fn add_namespace(&mut self, namespace: &str) -> NamespaceId {
        self.namespace_lookup.get_id_mut(namespace)
    }

    /// Look up prefix.
    pub fn prefix(&self, prefix: &str) -> Option<PrefixId> {
        self.prefix_lookup.get_id(prefix)
    }

    /// Add prefix.
    ///
    /// If the prefix already exists, return its id.
    pub fn add_prefix(&mut self, prefix: &str) -> PrefixId {
        self.prefix_lookup.get_id_mut(prefix)
    }

    /// Look up localname, namespace uri for name id
    ///
    /// If this name id is not in a namespace, the namespace uri is the
    /// empty string.
    pub fn name_ns_str(&self, name: NameId) -> (&str, &str) {
        let name = self.name_lookup.get_value(name);
        let namespace = self.namespace_lookup.get_value(name.namespace_id);
        (name.name.as_str(), namespace)
    }

    /// Look up namespace uri for namespace id
    ///
    /// An empty string slice indicates the no namespace.
    pub fn namespace_str(&self, namespace: NamespaceId) -> &str {
        let namespace = self.namespace_lookup.get_value(namespace);
        namespace
    }

    /// Look up string slice for prefix id
    ///
    /// If the prefix id is the empty prefix, the string slice is the empty string.
    pub fn prefix_str(&self, prefix: PrefixId) -> &str {
        let prefix = self.prefix_lookup.get_value(prefix);
        prefix
    }

    /// Creating missing prefixes.
    ///
    /// Due to creation or moving subtrees
    /// you can end up with XML elements or attributes
    /// that have names in a namespace without a prefix
    /// to define the namespace in its ancestors.
    ///
    /// This function creates the missing prefixes
    /// on the given node. The prefixes are named
    /// "n0", "n1", "n2", etc.
    pub fn create_missing_prefixes(&mut self, node: Node) -> Result<(), Error> {
        if !self.is_element(node) {
            return Err(Error::NotElement(node));
        }
        let mut fullname_serializer = FullnameSerializer::new(self);
        let mut missing_namespace_ids = HashSet::default();
        for edge in self.traverse(node) {
            match edge {
                NodeEdge::Start(node) => {
                    let element = self.element(node);
                    if let Some(element) = element {
                        fullname_serializer.push(&element.namespace_info.to_prefix);
                        let element_fullname = fullname_serializer.fullname(element.name_id);
                        if let Fullname::MissingPrefix(namespace_id) = element_fullname {
                            missing_namespace_ids.insert(namespace_id);
                        }
                        for name_id in element.attributes.keys() {
                            let attribute_fullname = fullname_serializer.fullname(*name_id);
                            if let Fullname::MissingPrefix(namespace_id) = attribute_fullname {
                                missing_namespace_ids.insert(namespace_id);
                            }
                        }
                    }
                }
                NodeEdge::End(node) => {
                    let element = self.element(node);
                    if let Some(element) = element {
                        fullname_serializer.pop(&element.namespace_info.to_prefix);
                    }
                }
            }
        }
        let mut prefixes_to_add = HashMap::default();
        for (i, namespace_id) in missing_namespace_ids.iter().enumerate() {
            let prefix = format!("n{}", i);
            let prefix_id = self.add_prefix(&prefix);
            prefixes_to_add.insert(prefix_id, namespace_id);
        }
        let value = self.element_mut(node).unwrap();
        for (prefix_id, namespace_id) in prefixes_to_add {
            value.namespace_info.add(prefix_id, *namespace_id);
        }
        Ok(())
    }

    /// Deduplicate namespaces.
    ///
    /// Any namespace definition lower down that
    /// defines a prefix for a namespace that is already known in an ancestor
    /// is removed.
    pub fn deduplicate_namespaces(&mut self, node: Node) {
        let mut fullname_serializer = FullnameSerializer::new(self);
        let mut fixup_nodes = Vec::new();
        // determine nodes we need to fix up
        for edge in self.traverse(node) {
            match edge {
                NodeEdge::Start(node) => {
                    let element = self.element(node);
                    if let Some(element) = element {
                        // if we already know a namespace, remove it
                        let to_remove = element
                            .prefixes()
                            .iter()
                            .filter_map(|(_, namespace_id)| {
                                if fullname_serializer.is_namespace_known(*namespace_id) {
                                    Some(*namespace_id)
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();
                        if !to_remove.is_empty() {
                            fixup_nodes.push((node, to_remove.clone()));
                        }
                        // we don't need to remove the fixed up prefixes because
                        // as duplicates they will definitely exist.
                        // In fact if we remove them first the push will fail to create
                        // a new entry in the namespace stack, as to_prefix can become empty
                        fullname_serializer.push(&element.namespace_info.to_prefix);
                    }
                }
                NodeEdge::End(node) => {
                    let element = self.element(node);
                    if let Some(element) = element {
                        // to_prefix is only used to determine whether to pop
                        // so should be okay to send here
                        fullname_serializer.pop(&element.namespace_info.to_prefix);
                    }
                }
            }
        }
        // now actually fix up the nodes, removing superfluous namespaces
        for (node, to_remove) in fixup_nodes {
            let element = self.element_mut(node).unwrap();
            for namespace_id in to_remove {
                element.namespace_info.remove_by_namespace_id(namespace_id)
            }
        }
    }

    pub(crate) fn to_namespace_in_scope(&self, node: Node) -> ToNamespace {
        let mut to_namespace = ToNamespace::new();
        for ancestor in self.ancestors(node) {
            let element = self.element(ancestor);
            if let Some(element) = element {
                for (prefix_id, namespace_id) in element.prefixes() {
                    // prefixes defined later override those defined earlier
                    if to_namespace.contains_key(prefix_id) {
                        continue;
                    }
                    to_namespace.insert(*prefix_id, *namespace_id);
                }
            }
        }
        to_namespace
    }

    pub(crate) fn base_to_namespace(&self) -> ToNamespace {
        let mut to_namespace = ToNamespace::new();
        to_namespace.insert(self.xml_prefix_id, self.xml_namespace_id);
        to_namespace
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vector_map::VecMap;

    #[test]
    fn test_prefixes_in_scope() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<doc xmlns:foo="http://example.com"><a><b xmlns:foo="http://example.com/foo" xmlns:bar="http://example.com/bar" /></a></doc>"#)
            .unwrap();
        let doc_el = xot.document_element(root).unwrap();
        let a = xot.first_child(doc_el).unwrap();
        let b = xot.first_child(a).unwrap();

        let foo = xot.prefix("foo").unwrap();
        let ns = xot.namespace("http://example.com").unwrap();
        let ns_foo = xot.namespace("http://example.com/foo").unwrap();
        let ns_bar = xot.namespace("http://example.com/bar").unwrap();
        let bar = xot.prefix("bar").unwrap();

        assert_eq!(
            xot.to_namespace_in_scope(doc_el),
            VecMap::from_iter(vec![(foo, ns)])
        );

        assert_eq!(
            xot.to_namespace_in_scope(a),
            VecMap::from_iter(vec![(foo, ns)])
        );

        assert_eq!(
            xot.to_namespace_in_scope(b),
            VecMap::from_iter(vec![(foo, ns_foo), (bar, ns_bar)])
        );
    }
}
