use ahash::{HashMap, HashSet};

use crate::access::NodeEdge;
use crate::error::Error;
use crate::name::{Name, NameId};
use crate::namespace::{Namespace, NamespaceId};
use crate::prefix::{Prefix, PrefixId};
use crate::serialize::{Fullname, FullnameSerializer};
use crate::xmldata::{Node, XmlData};
use crate::xmlvalue::ToPrefix;

/// Creation and lookup of names, namespaces and prefixes.
impl XmlData {
    /// Look up name without a namespace.
    pub fn name(&self, name: &str) -> Option<NameId> {
        self.name_ns(name, self.no_namespace_id)
    }

    /// Add name without a namespace.
    /// If the name already exists, return its id.
    pub fn add_name(&mut self, name: &str) -> NameId {
        self.add_name_ns(name, self.no_namespace_id)
    }

    /// Look up name with a namespace.
    pub fn name_ns(&self, name: &str, namespace_id: NamespaceId) -> Option<NameId> {
        self.name_lookup
            .get_id(Name::new(name.to_string(), namespace_id))
    }

    /// Add name with a namespace.
    /// If the name already exists, return its id.
    pub fn add_name_ns(&mut self, name: &str, namespace_id: NamespaceId) -> NameId {
        self.name_lookup
            .get_id_mut(Name::new(name.to_string(), namespace_id))
    }

    /// Look up namespace.
    pub fn namespace(&self, namespace: &str) -> Option<NamespaceId> {
        self.namespace_lookup
            .get_id(Namespace::new(namespace.to_string()))
    }

    /// Add namespace.
    /// If the namespace already exists, return its id.
    pub fn add_namespace(&mut self, namespace: &str) -> NamespaceId {
        self.namespace_lookup
            .get_id_mut(Namespace::new(namespace.to_string()))
    }

    /// Look up prefix.
    pub fn prefix(&self, prefix: &str) -> Option<PrefixId> {
        self.prefix_lookup.get_id(Prefix::new(prefix.to_string()))
    }

    /// Add prefix.
    /// If the prefix already exists, return its id.
    pub fn add_prefix(&mut self, prefix: &str) -> PrefixId {
        self.prefix_lookup
            .get_id_mut(Prefix::new(prefix.to_string()))
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

    pub(crate) fn to_prefix_seen(&self, node: Node) -> ToPrefix {
        let mut fullname_serializer = FullnameSerializer::new(self);
        let mut to_prefix = ToPrefix::new();
        for edge in self.traverse(node) {
            match edge {
                NodeEdge::Start(sub_node) => {
                    let element = self.element(node);
                    if let Some(element) = element {
                        fullname_serializer.push(&element.namespace_info.to_prefix);
                        if node == sub_node {
                            to_prefix = fullname_serializer.top().clone();
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
        to_prefix
    }

    // deduplicate namespaces
    // if a namespace is used by multiple prefixes, use the first one
    // rename the names of the elements and attributes to use the first prefix
}
