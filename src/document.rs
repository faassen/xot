use id_tree::{NodeId, NodeIdError, Tree};
use std::fmt::{Debug, Formatter};

use crate::error::Error;
use crate::name::{NameId, NameLookup};
use crate::namespace::{NamespaceId, NamespaceLookup};
use crate::prefix::{PrefixId, PrefixLookup};
use crate::xmlnode::XmlNode;

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
