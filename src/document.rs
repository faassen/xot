use indextree::{Arena, Node, NodeEdge, NodeId};
use std::fmt::Debug;

use crate::error::Error;
use crate::name::{NameId, NameLookup};
use crate::namespace::{Namespace, NamespaceId, NamespaceLookup};
use crate::prefix::{Prefix, PrefixId, PrefixLookup};
use crate::xmlnode::XmlNode;

pub type XmlArena = Arena<XmlNode>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct XmlNodeId(NodeId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum XmlNodeEdge {
    Start(XmlNodeId),
    End(XmlNodeId),
}

pub struct XmlData {
    pub arena: XmlArena,
    pub(crate) namespace_lookup: NamespaceLookup,
    pub(crate) prefix_lookup: PrefixLookup,
    pub(crate) name_lookup: NameLookup,
    pub(crate) no_namespace_id: NamespaceId,
    pub(crate) empty_prefix_id: PrefixId,
}

impl XmlData {
    pub fn new() -> Self {
        let mut namespace_lookup = NamespaceLookup::new();
        let no_namespace_id = namespace_lookup.get_id(Namespace::new("".into()));
        let mut prefix_lookup = PrefixLookup::new();
        let empty_prefix_id = prefix_lookup.get_id(Prefix::new("".into()));
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

impl Default for XmlData {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Document {
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

impl Document {
    pub fn root_node_id(&self) -> NodeId {
        self.tree
    }

    // #[inline]
    // pub fn arena(&self) -> &XmlArena {
    //     &self.data.arena
    // }

    // #[inline]
    // pub fn arena_mut(&mut self) -> &mut XmlArena<'a> {
    //     &mut self.data.arena
    // }

    // #[inline]
    // pub fn node(&'a self, node_id: NodeId) -> &'a Node<XmlNode<'a>> {
    //     &self.arena()[node_id]
    // }

    // #[inline]
    // pub(crate) fn node_mut(&'a mut self, node_id: XmlNodeId) -> &'a mut Node<XmlNode<'a>> {
    //     &mut self.arena_mut()[node_id.0]
    // }

    // #[inline]
    // pub fn xml_node(&self, node_id: NodeId) -> &XmlNode {
    //     self.data.arena[node_id].get()
    // }

    // #[inline]
    // pub fn xml_node_mut(&mut self, node_id: NodeId) -> &'a mut XmlNode {
    //     self.data.arena[node_id].get_mut()
    // }

    // pub fn parent(&self, node_id: XmlNodeId) -> Option<XmlNodeId> {
    //     self.arena()[node_id.0].parent().map(XmlNodeId)
    // }

    // pub fn first_child(&self, node_id: XmlNodeId) -> Option<XmlNodeId> {
    //     self.arena()[node_id.0].first_child().map(XmlNodeId)
    // }

    // pub fn last_child(&self, node_id: XmlNodeId) -> Option<XmlNodeId> {
    //     self.arena()[node_id.0].last_child().map(XmlNodeId)
    // }

    // pub fn next_sibling(&self, node_id: XmlNodeId) -> Option<XmlNodeId> {
    //     self.arena()[node_id.0].next_sibling().map(XmlNodeId)
    // }

    // pub fn previous_sibling(&self, node_id: XmlNodeId) -> Option<XmlNodeId> {
    //     self.arena()[node_id.0].previous_sibling().map(XmlNodeId)
    // }

    // pub fn ancestors(&self, node_id: XmlNodeId) -> impl Iterator<Item = XmlNodeId> + '_ {
    //     node_id.0.ancestors(self.arena()).map(XmlNodeId)
    // }

    // pub fn append(&mut self, parent: XmlNodeId, child: XmlNodeId) -> Result<(), Error> {
    //     let xml_node = self.xml_node(parent);
    //     if matches!(xml_node, XmlNode::Root | XmlNode::Element(_)) {
    //         // XXX also check whether prefixes are valid
    //         parent.0.checked_append(child.0, self.arena_mut())?;
    //         Ok(())
    //     } else {
    //         Err(Error::InvalidOperation(
    //             "Can only append to elements or document root".into(),
    //         ))
    //     }
    // }

    // pub fn insert_after(
    //     &mut self,
    //     node_id: XmlNodeId,
    //     new_sibling: XmlNodeId,
    // ) -> Result<(), Error> {
    //     let xml_node = self.xml_node(node_id);
    //     if !matches!(xml_node, XmlNode::Root) {
    //         node_id
    //             .0
    //             .checked_insert_after(new_sibling.0, self.arena_mut())?;
    //         Ok(())
    //     } else {
    //         Err(Error::InvalidOperation(
    //             "Cannot insert after document root".into(),
    //         ))
    //     }
    // }

    // pub fn insert_before(
    //     &mut self,
    //     node_id: XmlNodeId,
    //     new_sibling: XmlNodeId,
    // ) -> Result<(), Error> {
    //     let xml_node = self.xml_node(node_id);
    //     if !matches!(xml_node, XmlNode::Root) {
    //         node_id
    //             .0
    //             .checked_insert_before(new_sibling.0, self.arena_mut())?;
    //         Ok(())
    //     } else {
    //         Err(Error::InvalidOperation(
    //             "Cannot insert before document root".into(),
    //         ))
    //     }
    // }

    // pub fn prepend(&mut self, parent: XmlNodeId, child: XmlNodeId) -> Result<(), Error> {
    //     let xml_node = self.xml_node(parent);
    //     if matches!(xml_node, XmlNode::Root | XmlNode::Element(_)) {
    //         parent.0.checked_prepend(child.0, self.arena_mut())?;
    //         Ok(())
    //     } else {
    //         Err(Error::InvalidOperation(
    //             "Can only prepend to elements or document root".into(),
    //         ))
    //     }
    // }

    // pub fn children(&self, node_id: XmlNodeId) -> impl Iterator<Item = XmlNodeId> + '_ {
    //     node_id.0.children(self.arena()).map(XmlNodeId)
    // }

    // pub fn reverse_children(&self, node_id: XmlNodeId) -> impl Iterator<Item = XmlNodeId> + '_ {
    //     node_id.0.reverse_children(self.arena()).map(XmlNodeId)
    // }

    // pub fn descendants(&self, node_id: XmlNodeId) -> impl Iterator<Item = XmlNodeId> + '_ {
    //     node_id.0.descendants(self.arena()).map(XmlNodeId)
    // }

    // pub fn detach(&mut self, node_id: XmlNodeId) {
    //     node_id.0.detach(self.arena_mut());
    // }

    // pub fn following_siblings(&self, node_id: XmlNodeId) -> impl Iterator<Item = XmlNodeId> + '_ {
    //     node_id.0.following_siblings(self.arena()).map(XmlNodeId)
    // }

    // pub fn preceding_siblings(&self, node_id: XmlNodeId) -> impl Iterator<Item = XmlNodeId> + '_ {
    //     node_id.0.preceding_siblings(self.arena()).map(XmlNodeId)
    // }

    // pub fn is_removed(&self, node_id: XmlNodeId) -> bool {
    //     self.arena()[node_id.0].is_removed()
    // }

    // pub fn remove(&mut self, node_id: XmlNodeId) {
    //     node_id.0.remove_subtree(self.arena_mut());
    // }

    // pub fn traverse(&self, node_id: XmlNodeId) -> impl Iterator<Item = XmlNodeEdge> + '_ {
    //     node_id.0.traverse(self.arena()).map(|edge| match edge {
    //         NodeEdge::Start(node_id) => XmlNodeEdge::Start(XmlNodeId(node_id)),
    //         NodeEdge::End(node_id) => XmlNodeEdge::End(XmlNodeId(node_id)),
    //     })
    // }

    // pub fn reverse_traverse(&self, node_id: XmlNodeId) -> impl Iterator<Item = XmlNodeEdge> + '_ {
    //     node_id
    //         .0
    //         .reverse_traverse(self.arena())
    //         .map(|edge| match edge {
    //             NodeEdge::Start(node_id) => XmlNodeEdge::Start(XmlNodeId(node_id)),
    //             NodeEdge::End(node_id) => XmlNodeEdge::End(XmlNodeId(node_id)),
    //         })
    // }

    // XXX probably break this into convenience methods
    // to lookup prefix. Getting the prefix is only handy when doing
    // tree manipulation in rare cases, as usually namespace is
    // fine. During serialization we use a special stack for
    // performance reasons.
    // fn fullname(&self, node_id: NodeId, name_id: NameId) -> Result<String, Error> {
    //     let name = self.data.name_lookup.get_value(name_id);
    //     if name.namespace_id == self.data.no_namespace_id {
    //         return Ok(name.name.to_string());
    //     }
    //     // XXX this is relatively slow
    //     let prefix_id = prefix_by_namespace(node_id, name.namespace_id, &self.data.arena);
    //     // if prefix_id cannot be found, then that's an error: we have removed
    //     // a prefix declaration even though it is still in use
    //     let prefix_id = prefix_id.ok_or_else(|| {
    //         Error::NoPrefixForNamespace(
    //             self.data
    //                 .namespace_lookup
    //                 .get_value(name.namespace_id)
    //                 .to_string(),
    //         )
    //     })?;
    //     if prefix_id == self.data.empty_prefix_id {
    //         Ok(format!("{}", name.name))
    //     } else {
    //         let prefix = self.data.prefix_lookup.get_value(prefix_id);
    //         Ok(format!("{}:{}", prefix, name.name))
    //     }
    // }
}

// impl<'a> Debug for Document<'a> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         self.root_node_id()
//             .debug_pretty_print(&self.data.arena)
//             .fmt(f)
//     }
// }
