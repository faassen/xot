use ahash::{HashMap, HashSet};

use crate::access::NodeEdge;
use crate::error::Error;
use crate::name::{Name, NameId};
use crate::namespace::{Namespace, NamespaceId};
use crate::prefix::{Prefix, PrefixId};
use crate::serialize::{Fullname, FullnameSerializer};
use crate::xmldata::{Node, XmlData};

impl XmlData {
    // name & namespace
    pub fn name(&self, name: &str) -> Option<NameId> {
        self.name_ns(name, self.no_namespace_id)
    }

    pub fn add_name(&mut self, name: &str) -> NameId {
        self.add_name_ns(name, self.no_namespace_id)
    }

    pub fn name_ns(&self, name: &str, namespace_id: NamespaceId) -> Option<NameId> {
        self.name_lookup
            .get_id(Name::new(name.to_string(), namespace_id))
    }

    pub fn add_name_ns(&mut self, name: &str, namespace_id: NamespaceId) -> NameId {
        self.name_lookup
            .get_id_mut(Name::new(name.to_string(), namespace_id))
    }

    pub fn namespace(&self, namespace: &str) -> Option<NamespaceId> {
        self.namespace_lookup
            .get_id(Namespace::new(namespace.to_string()))
    }

    pub fn add_namespace(&mut self, namespace: &str) -> NamespaceId {
        self.namespace_lookup
            .get_id_mut(Namespace::new(namespace.to_string()))
    }

    pub fn prefix(&self, prefix: &str) -> Option<PrefixId> {
        self.prefix_lookup.get_id(Prefix::new(prefix.to_string()))
    }

    pub fn add_prefix(&mut self, prefix: &str) -> PrefixId {
        self.prefix_lookup
            .get_id_mut(Prefix::new(prefix.to_string()))
    }

    // For any namespace under node that does not have a prefix, create
    // a prefix for it and add it to the node
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
}
