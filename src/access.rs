use indextree::NodeEdge as IndexTreeNodeEdge;

use crate::error::Error;
use crate::xmlvalue::{Value, ValueType};
use crate::xotdata::{Node, Xot};

/// Node edges.
///
/// Used by [`Xot::traverse`] and [`Xot::reverse_traverse`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeEdge {
    Start(Node),
    End(Node),
}

/// ## Read-only access
impl Xot {
    /// Obtain the root element from the document root.
    /// Returns [`Error::NotRoot`](`crate::error::Error::NotRoot`) error if
    /// this is not the document root.
    pub fn document_element(&self, node: Node) -> Result<Node, Error> {
        if self.value_type(node) != ValueType::Root {
            return Err(Error::NotRoot(node));
        }
        for child in self.children(node) {
            if let Value::Element(_) = self.value(child) {
                return Ok(child);
            }
        }
        unreachable!("Document should always have a single root node")
    }

    /// Obtain top element,
    /// given anywhere in a tree.
    ///
    /// In an XML document this is the document element.
    /// In an XML fragment it's the top node of the
    /// fragment.
    pub fn top_element(&self, node: Node) -> Node {
        if self.value_type(node) == ValueType::Root {
            return self.document_element(node).unwrap();
        }
        let mut top = node;
        for ancestor in self.ancestors(node) {
            if let Value::Element(_) = self.value(ancestor) {
                top = ancestor;
            }
        }
        // XXX in a fragment this may not be an element.
        top
    }

    /// Check whether a node has been removed.
    pub fn is_removed(&self, node: Node) -> bool {
        self.arena()[node.get()].is_removed()
    }

    /// Get parent node.
    pub fn parent(&self, node: Node) -> Option<Node> {
        self.arena()[node.get()].parent().map(Node::new)
    }

    /// Get first child.
    pub fn first_child(&self, node: Node) -> Option<Node> {
        self.arena()[node.get()].first_child().map(Node::new)
    }

    /// Get last child.
    pub fn last_child(&self, node: Node) -> Option<Node> {
        self.arena()[node.get()].last_child().map(Node::new)
    }

    /// Get next sibling.
    pub fn next_sibling(&self, node: Node) -> Option<Node> {
        self.arena()[node.get()].next_sibling().map(Node::new)
    }

    /// Get previous sibling.
    pub fn previous_sibling(&self, node: Node) -> Option<Node> {
        self.arena()[node.get()].previous_sibling().map(Node::new)
    }

    /// Iterator over ancestor nodes, including this one.
    pub fn ancestors(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        node.get().ancestors(self.arena()).map(Node::new)
    }

    /// Iterator over the child nodes of this node.
    pub fn children(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        node.get().children(self.arena()).map(Node::new)
    }

    /// Iterator over the child nodes of this node, in reverse order.
    pub fn reverse_children(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        node.get().reverse_children(self.arena()).map(Node::new)
    }

    /// Iterator over of the descendants of this node,
    /// including this one. In document order (pre-order depth-first).
    pub fn descendants(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        node.get().descendants(self.arena()).map(Node::new)
    }

    /// Iterator over the following siblings of this node.
    pub fn following_siblings(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        node.get().following_siblings(self.arena()).map(Node::new)
    }

    /// Iterator over the preceding siblings of this node.
    pub fn preceding_siblings(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        node.get().preceding_siblings(self.arena()).map(Node::new)
    }

    /// Traverse over node edges.
    ///
    /// This useful to traverse the tree in document order
    /// iteratively without the need for recursion.
    ///
    /// For the tree `<a><b/></a>` this generates
    /// a [`NodeEdge::Start`] for `<a>`, then
    /// a [`NodeEdge::Start`] for `<b>`, immediately
    /// followed by a [`NodeEdge::End`] for `<b>`,
    /// and finally a [`NodeEdge::End`] for `<a>`.
    ///
    /// For value types other than element or root,
    /// the start and end always come as pairs without
    /// any intervening edges.
    pub fn traverse(&self, node: Node) -> impl Iterator<Item = NodeEdge> + '_ {
        node.get().traverse(self.arena()).map(|edge| match edge {
            IndexTreeNodeEdge::Start(node_id) => NodeEdge::Start(Node::new(node_id)),
            IndexTreeNodeEdge::End(node_id) => NodeEdge::End(Node::new(node_id)),
        })
    }

    /// Traverse over node edges in reverse order.
    ///
    /// Like [`Xot::traverse`] but in reverse order.
    pub fn reverse_traverse(&self, node: Node) -> impl Iterator<Item = NodeEdge> + '_ {
        node.get()
            .reverse_traverse(self.arena())
            .map(|edge| match edge {
                IndexTreeNodeEdge::Start(node_id) => NodeEdge::Start(Node::new(node_id)),
                IndexTreeNodeEdge::End(node_id) => NodeEdge::End(Node::new(node_id)),
            })
    }
}
