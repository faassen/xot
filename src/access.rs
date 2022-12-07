use indextree::NodeEdge as IndexTreeNodeEdge;

use crate::xmldata::{Node, XmlData};
use crate::xmlvalue::{Value, ValueType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeEdge {
    Start(Node),
    End(Node),
}

impl XmlData {
    pub fn root_element(&self, node: Node) -> Node {
        if self.value_type(node) != ValueType::Root {
            unreachable!("Can only obtain the root element for document root");
        }
        for child in self.children(node) {
            if let Value::Element(_) = self.value(child) {
                return child;
            }
        }
        unreachable!("Document should always have a single root node")
    }

    pub fn parent(&self, node: Node) -> Option<Node> {
        self.arena()[node.get()].parent().map(Node::new)
    }

    pub fn first_child(&self, node: Node) -> Option<Node> {
        self.arena()[node.get()].first_child().map(Node::new)
    }

    pub fn last_child(&self, node: Node) -> Option<Node> {
        self.arena()[node.get()].last_child().map(Node::new)
    }

    pub fn next_sibling(&self, node: Node) -> Option<Node> {
        self.arena()[node.get()].next_sibling().map(Node::new)
    }

    pub fn previous_sibling(&self, node: Node) -> Option<Node> {
        self.arena()[node.get()].previous_sibling().map(Node::new)
    }

    pub fn ancestors(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        node.get().ancestors(self.arena()).map(Node::new)
    }

    pub fn children(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        node.get().children(self.arena()).map(Node::new)
    }

    pub fn reverse_children(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        node.get().reverse_children(self.arena()).map(Node::new)
    }

    pub fn descendants(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        node.get().descendants(self.arena()).map(Node::new)
    }

    pub fn following_siblings(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        node.get().following_siblings(self.arena()).map(Node::new)
    }

    pub fn preceding_siblings(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        node.get().preceding_siblings(self.arena()).map(Node::new)
    }

    pub fn is_removed(&self, node: Node) -> bool {
        self.arena()[node.get()].is_removed()
    }

    pub fn traverse(&self, node: Node) -> impl Iterator<Item = NodeEdge> + '_ {
        node.get().traverse(self.arena()).map(|edge| match edge {
            IndexTreeNodeEdge::Start(node_id) => NodeEdge::Start(Node::new(node_id)),
            IndexTreeNodeEdge::End(node_id) => NodeEdge::End(Node::new(node_id)),
        })
    }

    pub fn reverse_traverse(&self, node: Node) -> impl Iterator<Item = NodeEdge> + '_ {
        node.get()
            .reverse_traverse(self.arena())
            .map(|edge| match edge {
                IndexTreeNodeEdge::Start(node_id) => NodeEdge::Start(Node::new(node_id)),
                IndexTreeNodeEdge::End(node_id) => NodeEdge::End(Node::new(node_id)),
            })
    }
}
