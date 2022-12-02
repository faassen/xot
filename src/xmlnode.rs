use id_tree::{NodeId, NodeIdError, Tree};
use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use vector_map::VecMap;

use crate::error::Error;
use crate::name::{Name, NameId, NameLookup};
use crate::namespace::{Namespace, NamespaceId, NamespaceLookup};
use crate::prefix::{PrefixId, PrefixLookup};

#[derive(Debug)]
pub enum XmlNode<'a> {
    Element(Element<'a>),
    Text(Cow<'a, str>),
}

pub(crate) type Attributes<'a> = VecMap<NameId, Cow<'a, str>>;
pub(crate) type Prefixes = VecMap<PrefixId, NamespaceId>;

#[derive(Debug)]
pub(crate) struct NamespaceInfo {
    pub(crate) to_namespace: VecMap<PrefixId, NamespaceId>,
    pub(crate) to_prefix: VecMap<NamespaceId, PrefixId>,
}

impl NamespaceInfo {
    pub(crate) fn new() -> Self {
        NamespaceInfo {
            to_namespace: VecMap::new(),
            to_prefix: VecMap::new(),
        }
    }

    pub(crate) fn add(&mut self, prefix_id: PrefixId, namespace_id: NamespaceId) {
        self.to_namespace.insert(prefix_id, namespace_id);
        self.to_prefix.insert(namespace_id, prefix_id);
    }
}

#[derive(Debug)]
pub struct Element<'a> {
    pub(crate) name_id: NameId,
    pub(crate) attributes: Attributes<'a>,
    pub(crate) namespace_info: NamespaceInfo,
}

impl<'a> Element<'a> {
    pub(crate) fn new(name_id: NameId) -> Self {
        Element {
            name_id,
            attributes: VecMap::new(),
            namespace_info: NamespaceInfo::new(),
        }
    }

    pub fn get_attributes(&'a self) -> &'a Attributes<'a> {
        &self.attributes
    }

    pub fn get_attributes_mut(&'a mut self) -> &'a mut Attributes<'a> {
        &mut self.attributes
    }

    // pub fn get_prefixes(&'a self) -> &Prefixes {
    //     &self.prefixes
    // }

    // pub fn get_prefixes_mut(&'a mut self) -> &mut Prefixes {
    //     &mut self.prefixes
    // }

    // pub(crate) fn add_prefix(&'a mut self, prefix_id: PrefixId, namespace_id: NamespaceId) {
    //     self.prefixes.insert(prefix_id, namespace_id);
    // }
}

pub(crate) type XmlTree<'a> = Tree<XmlNode<'a>>;

pub struct Document<'a> {
    pub(crate) namespace_lookup: NamespaceLookup<'a>,
    pub(crate) prefix_lookup: PrefixLookup<'a>,
    pub(crate) name_lookup: NameLookup<'a>,
    pub(crate) tree: XmlTree<'a>,
    pub(crate) no_namespace_id: NamespaceId,
}

pub(crate) fn prefix_by_namespace(
    tree: &XmlTree,
    node_id: &NodeId,
    namespace_id: NamespaceId,
) -> Result<Option<PrefixId>, NodeIdError> {
    for ancestor in tree.ancestors(node_id)? {
        let xml_node = ancestor.data();
        if let XmlNode::Element(element) = xml_node {
            if let Some(prefix_id) = element.namespace_info.to_prefix.get(&namespace_id) {
                return Ok(Some(*prefix_id));
            }
        }
    }
    Ok(None)
}

pub(crate) fn namespace_by_prefix(
    tree: &XmlTree,
    node_id: &NodeId,
    prefix_id: PrefixId,
) -> Result<Option<NamespaceId>, NodeIdError> {
    for ancestor in tree.ancestors(node_id)? {
        let xml_node = ancestor.data();
        if let XmlNode::Element(element) = xml_node {
            if let Some(namespace_id) = element.namespace_info.to_namespace.get(&prefix_id) {
                return Ok(Some(*namespace_id));
            }
        }
    }
    Ok(None)
}

impl<'a> Document<'a> {
    pub fn root_node_id(&self) -> Option<&NodeId> {
        self.tree.root_node_id()
    }

    pub(crate) fn fullname(&self, node_id: &NodeId, name_id: NameId) -> Result<String, Error> {
        let name = self.name_lookup.get_value(name_id);
        if name.namespace_id == self.no_namespace_id {
            return Ok(name.name.to_string());
        }
        // XXX this is relatively slow
        let prefix_id = prefix_by_namespace(&self.tree, node_id, name.namespace_id)?;
        // if prefix_id cannot be found, then that's an error: we have removed
        // a prefix declaration even though it is still in use
        let prefix_id = prefix_id.ok_or_else(|| {
            Error::NoPrefixForNamespace(
                self.namespace_lookup
                    .get_value(name.namespace_id)
                    .to_string(),
            )
        })?;
        let prefix = self.prefix_lookup.get_value(prefix_id);
        Ok(format!("{}:{}", prefix, name.name))
    }
}

impl<'a> Debug for Document<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        let mut s = String::new();
        self.tree.write_formatted(&mut s)?;
        f.write_str(&s)
    }
}
