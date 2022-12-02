use indextree::{Arena, NodeId};

use crate::error::Error;
use crate::name::{NameId, NameLookup};
use crate::namespace::{NamespaceId, NamespaceLookup};
use crate::prefix::{PrefixId, PrefixLookup};
use crate::xmlnode::XmlNode;

pub type XmlArena<'a> = Arena<XmlNode<'a>>;

pub struct Document<'a> {
    pub(crate) arena: &'a mut XmlArena<'a>,
    pub(crate) namespace_lookup: NamespaceLookup<'a>,
    pub(crate) prefix_lookup: PrefixLookup<'a>,
    pub(crate) name_lookup: NameLookup<'a>,
    pub(crate) tree: NodeId,
    pub(crate) no_namespace_id: NamespaceId,
}

pub(crate) fn prefix_by_namespace(
    node_id: NodeId,
    namespace_id: NamespaceId,
    arena: &XmlArena,
) -> Option<PrefixId> {
    for ancestor in node_id.ancestors(arena) {
        let xml_node = arena.get(ancestor).unwrap().get();
        if let XmlNode::Element(element) = xml_node {
            if let Some(prefix_id) = element.namespace_info.to_prefix.get(&namespace_id) {
                return Some(*prefix_id);
            }
        }
    }
    None
}

pub(crate) fn namespace_by_prefix(
    node_id: NodeId,
    prefix_id: PrefixId,
    arena: &XmlArena,
) -> Option<NamespaceId> {
    for ancestor in node_id.ancestors(arena) {
        let xml_node = arena.get(ancestor).unwrap().get();
        if let XmlNode::Element(element) = xml_node {
            if let Some(namespace_id) = element.namespace_info.to_namespace.get(&prefix_id) {
                return Some(*namespace_id);
            }
        }
    }
    None
}

impl<'a> Document<'a> {
    pub fn root_node_id(&self) -> NodeId {
        self.tree
    }

    pub(crate) fn fullname(&self, node_id: NodeId, name_id: NameId) -> Result<String, Error> {
        let name = self.name_lookup.get_value(name_id);
        if name.namespace_id == self.no_namespace_id {
            return Ok(name.name.to_string());
        }
        // XXX this is relatively slow
        let prefix_id = prefix_by_namespace(node_id, name.namespace_id, &self.arena);
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
