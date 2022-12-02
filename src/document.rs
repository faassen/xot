use indextree::{Arena, NodeId};
use std::fmt::Debug;

use crate::error::Error;
use crate::name::{NameId, NameLookup};
use crate::namespace::{Namespace, NamespaceId, NamespaceLookup};
use crate::prefix::{Prefix, PrefixId, PrefixLookup};
use crate::xmlnode::XmlNode;

pub type XmlArena<'a> = Arena<XmlNode<'a>>;

pub struct XmlData<'a> {
    pub(crate) arena: XmlArena<'a>,
    pub(crate) namespace_lookup: NamespaceLookup<'a>,
    pub(crate) prefix_lookup: PrefixLookup<'a>,
    pub(crate) name_lookup: NameLookup<'a>,
    pub(crate) no_namespace_id: NamespaceId,
    pub(crate) empty_prefix_id: PrefixId,
}

impl<'a> XmlData<'a> {
    pub fn new() -> Self {
        let mut namespace_lookup = NamespaceLookup::new();
        let no_namespace_id = namespace_lookup.get_id(Namespace::new(""));
        let mut prefix_lookup = PrefixLookup::new();
        let empty_prefix_id = prefix_lookup.get_id(Prefix::new(""));
        XmlData {
            arena: XmlArena::new(),
            namespace_lookup,
            prefix_lookup,
            name_lookup: NameLookup::new(),
            no_namespace_id,
            empty_prefix_id,
        }
    }
}

impl<'a> Default for XmlData<'a> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Document<'a> {
    pub(crate) data: &'a mut XmlData<'a>,
    pub(crate) tree: NodeId,
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
        let name = self.data.name_lookup.get_value(name_id);
        if name.namespace_id == self.data.no_namespace_id {
            return Ok(name.name.to_string());
        }
        // XXX this is relatively slow
        let prefix_id = prefix_by_namespace(node_id, name.namespace_id, &self.data.arena);
        // if prefix_id cannot be found, then that's an error: we have removed
        // a prefix declaration even though it is still in use
        let prefix_id = prefix_id.ok_or_else(|| {
            Error::NoPrefixForNamespace(
                self.data
                    .namespace_lookup
                    .get_value(name.namespace_id)
                    .to_string(),
            )
        })?;
        if prefix_id == self.data.empty_prefix_id {
            Ok(format!("{}", name.name))
        } else {
            let prefix = self.data.prefix_lookup.get_value(prefix_id);
            Ok(format!("{}:{}", prefix, name.name))
        }
    }
}

impl<'a> Debug for Document<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.root_node_id()
            .debug_pretty_print(&self.data.arena)
            .fmt(f)
    }
}
