use indextree::NodeEdge as IndexTreeNodeEdge;

use crate::error::Error;
use crate::levelorder::{level_order_traverse, LevelOrder};
use crate::nodemap::{Attributes, Namespaces};
use crate::xmlvalue::{Value, ValueCategory, ValueType};
use crate::xotdata::{Node, Xot};
use crate::{MutableAttributes, MutableNamespaces};

/// Node edges.
///
/// Used by [`Xot::traverse`] and [`Xot::reverse_traverse`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeEdge {
    /// The start edge of a node. In case of an element
    /// this is the start tag. In case of root
    /// the start of the document.
    Start(Node),
    /// The end edge of a node. In case of an element
    /// this is the end tag. In case of root the end
    /// of the document. For any other values, the
    /// end edge occurs immediately after the start
    /// edge.  
    End(Node),
}

/// ## Read-only access
impl Xot {
    /// Obtain the root element from the document root.
    /// Returns [`Error::NotRoot`](`crate::error::Error::NotRoot`) error if
    /// this is not the document root.
    ///
    /// ```rust
    /// let mut xot = xot::Xot::new();
    ///
    /// let root = xot.parse("<p>Example</p>").unwrap();
    ///
    /// let doc_el = xot.document_element(root).unwrap();
    ///
    /// // Check that we indeed have the `p` element
    /// let p_name = xot.name("p").unwrap();
    /// assert_eq!(xot.element(doc_el).unwrap().name(), p_name);
    /// ```
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

    /// Obtain top element, given node anywhere in a tree.
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
    ///
    /// This can happen because you removed it explicitly, or because you held
    /// on to a reference and the node was replaced using [`Xot::replace`], or
    /// unwrapped using [`Xot::element_unwrap`].
    ///
    /// ```rust
    /// let mut xot = xot::Xot::new();
    ///
    /// let root = xot.parse("<p>Example</p>").unwrap();
    /// let p = xot.document_element(root).unwrap();
    /// let text = xot.first_child(p).unwrap();
    /// xot.remove(text);
    /// assert_eq!(xot.to_string(root).unwrap(), "<p/>");
    /// assert!(xot.is_removed(text));
    /// ```
    pub fn is_removed(&self, node: Node) -> bool {
        self.arena()[node.get()].is_removed()
    }

    /// Get parent node.
    ///
    /// Returns [`None`] if this is the root node or if the node is unattached
    /// to a document.
    ///
    /// Attribute and namespace nodes have a parent, even though they aren't
    /// children of the element they are in.
    ///
    /// ```rust
    /// let mut xot = xot::Xot::new();
    /// let root = xot.parse("<p>Example</p>").unwrap();
    /// let p = xot.document_element(root).unwrap();
    /// let text = xot.first_child(p).unwrap();
    /// assert_eq!(xot.parent(text), Some(p));
    /// assert_eq!(xot.parent(p), Some(root));
    /// assert_eq!(xot.parent(root), None);
    /// ```
    pub fn parent(&self, node: Node) -> Option<Node> {
        self.arena()[node.get()].parent().map(Node::new)
    }

    pub(crate) fn all_children(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        node.get().children(&self.arena).map(Node::new)
    }

    fn normal_children(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        node.get()
            .children(&self.arena)
            .skip_while(|n| !self.arena[*n].get().is_normal())
            .map(Node::new)
    }

    /// Attributes accessor.
    ///
    /// Returns a map of attributes.
    pub fn attributes(&self, node: Node) -> Attributes {
        Attributes::new(self, node)
    }

    /// Mutable attributes accessor
    pub fn attributes_mut(&mut self, node: Node) -> MutableAttributes {
        MutableAttributes::new(self, node)
    }

    /// Namespaces accessor.
    ///
    /// Returns a map of namespaces.
    pub fn namespaces(&self, node: Node) -> Namespaces {
        Namespaces::new(self, node)
    }

    /// Mutable namespaces accessor.
    pub fn namespaces_mut(&mut self, node: Node) -> MutableNamespaces {
        MutableNamespaces::new(self, node)
    }

    /// Get first child.
    ///
    /// Returns [`None`] if there are no children.
    ///
    /// ```rust
    /// let mut xot = xot::Xot::new();
    /// let root = xot.parse("<p>Example</p>").unwrap();
    /// let p = xot.document_element(root).unwrap();
    /// let text = xot.first_child(p).unwrap();
    /// assert_eq!(xot.first_child(root), Some(p));
    /// assert_eq!(xot.first_child(p), Some(text));
    /// assert_eq!(xot.first_child(text), None);
    /// ```
    pub fn first_child(&self, node: Node) -> Option<Node> {
        self.normal_children(node).next()
    }

    /// Get last child.
    ///
    /// Returns [`None`] if there are no children.
    pub fn last_child(&self, node: Node) -> Option<Node> {
        let last_child = self.arena[node.get()].last_child()?;
        if self.arena[last_child].get().is_normal() {
            Some(Node::new(last_child))
        } else {
            None
        }
    }

    /// Get next sibling.
    ///
    /// Returns [`None`] if there is no next sibling.
    ///
    /// For normal child nodes, gives the next child.
    ///
    /// For namespace and attribute nodes, gives the next namespace or
    /// attribute in definition order.
    ///
    /// ```rust
    /// let mut xot = xot::Xot::new();
    /// let root = xot.parse("<p><a/><b/></p>").unwrap();
    /// let p = xot.document_element(root).unwrap();
    /// let a = xot.first_child(p).unwrap();
    /// let b = xot.next_sibling(a).unwrap();
    /// assert_eq!(xot.next_sibling(b), None);
    /// ```
    pub fn next_sibling(&self, node: Node) -> Option<Node> {
        let current_category = self.arena[node.get()].get().value_category();
        let next_sibling = self.arena[node.get()].next_sibling()?;
        let next_category = self.arena[next_sibling].get().value_category();
        if current_category != next_category {
            return None;
        }
        Some(Node::new(next_sibling))
    }

    /// Get previous sibling.
    ///
    /// Returns [`None`] if there is no previous sibling.
    pub fn previous_sibling(&self, node: Node) -> Option<Node> {
        let current_category = self.arena[node.get()].get().value_category();
        let previous_sibling = self.arena[node.get()].previous_sibling()?;
        let previous_category = self.arena[previous_sibling].get().value_category();
        if current_category != previous_category {
            return None;
        }
        Some(Node::new(previous_sibling))
    }

    /// Iterator over ancestor nodes, including this one.
    ///
    /// Namespace and attribute node have ancestors, even though
    /// they aren't the child of the element they are in.
    ///
    /// ```rust
    /// let mut xot = xot::Xot::new();
    ///
    /// let root = xot.parse("<a><b><c/></b></a>").unwrap();
    /// let a = xot.document_element(root).unwrap();
    /// let b = xot.first_child(a).unwrap();
    /// let c = xot.first_child(b).unwrap();
    ///
    /// let ancestors = xot.ancestors(c).collect::<Vec<_>>();
    /// assert_eq!(ancestors, vec![c, b, a, root]);
    /// ```
    pub fn ancestors(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        node.get().ancestors(self.arena()).map(Node::new)
    }

    /// Iterator over the child nodes of this node.
    ///
    /// Namespace and attribute nodes aren't consider child nodes even
    /// if they have a parent element.
    ///
    /// ```rust
    /// let mut xot = xot::Xot::new();
    /// let root = xot.parse("<p><a/><b/></p>").unwrap();
    /// let p = xot.document_element(root).unwrap();
    /// let a = xot.first_child(p).unwrap();
    /// let b = xot.next_sibling(a).unwrap();
    /// let children = xot.children(p).collect::<Vec<_>>();
    ///
    /// assert_eq!(children, vec![a, b]);
    /// ```
    pub fn children(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        self.normal_children(node)
    }

    /// Get index of child.
    ///
    /// Returns [`None`] if the node is not a child of this node, so
    /// does not apply to namespace or attribute nodes.
    ///
    /// ```rust
    /// let mut xot = xot::Xot::new();
    /// let root = xot.parse("<p><a/><b/></p>").unwrap();
    /// let p = xot.document_element(root).unwrap();
    /// let a = xot.first_child(p).unwrap();
    /// let b = xot.next_sibling(a).unwrap();
    /// assert_eq!(xot.child_index(p, a), Some(0));
    /// assert_eq!(xot.child_index(p, b), Some(1));
    /// assert_eq!(xot.child_index(a, b), None);
    /// ```
    pub fn child_index(&self, parent: Node, child: Node) -> Option<usize> {
        if self.parent(child) != Some(parent) {
            return None;
        }
        self.normal_children(parent).position(|n| n == child)
    }

    /// Iterator over the child nodes of this node, in reverse order.
    pub fn reverse_children(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        node.get()
            .reverse_children(self.arena())
            .take_while(|n| self.arena[*n].get().is_normal())
            .map(Node::new)
    }

    fn normal_filter(&self) -> impl Fn(&indextree::NodeId) -> bool + '_ {
        |node_id| self.arena[*node_id].get().is_normal()
    }

    fn category_filter(&self, category: ValueCategory) -> impl Fn(&indextree::NodeId) -> bool + '_ {
        move |node_id| self.arena[*node_id].get().value_category() == category
    }

    /// Iterator over of the descendants of this node,
    /// including this one. In document order (pre-order depth-first).
    ///
    /// Namespace and attribute nodes aren't included as descendants.
    ///
    /// ```rust
    /// let mut xot = xot::Xot::new();
    /// let root = xot.parse("<a><b><c/></b></a>").unwrap();
    /// let a = xot.document_element(root).unwrap();
    /// let b = xot.first_child(a).unwrap();
    /// let c = xot.first_child(b).unwrap();
    ///
    /// let descendants = xot.descendants(a).collect::<Vec<_>>();
    /// assert_eq!(descendants, vec![a, b, c]);
    /// ```
    pub fn descendants(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        node.get()
            .descendants(self.arena())
            .filter(self.normal_filter())
            .map(Node::new)
    }

    /// Iterator over the following siblings of this node, including this one.
    ///
    /// In case of namespace or attribute nodes, includes the following sibling
    /// namespace or attribute nodes.
    ///
    /// ```rust
    /// let mut xot = xot::Xot::new();
    /// let root = xot.parse("<p><a/><b/><c/></p>").unwrap();
    /// let p = xot.document_element(root).unwrap();
    /// let a = xot.first_child(p).unwrap();
    /// let b = xot.next_sibling(a).unwrap();
    /// let c = xot.next_sibling(b).unwrap();
    /// let siblings = xot.following_siblings(a).collect::<Vec<_>>();
    /// assert_eq!(siblings, vec![a, b, c]);
    /// let siblings = xot.following_siblings(b).collect::<Vec<_>>();
    /// assert_eq!(siblings, vec![b, c]);
    /// ```
    pub fn following_siblings(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        let current_category = self.arena[node.get()].get().value_category();
        node.get()
            .following_siblings(self.arena())
            .filter(self.category_filter(current_category))
            .map(Node::new)
    }

    /// Iterator over the preceding siblings of this node.
    pub fn preceding_siblings(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        let current_category = self.arena[node.get()].get().value_category();
        node.get()
            .preceding_siblings(self.arena())
            .filter(self.category_filter(current_category))
            .map(Node::new)
    }

    /// Following nodes in document order
    ///
    /// These are nodes that come after this node in document order,
    /// without that node itself, its ancestors, or its descendants.
    ///
    /// Does not include namespace or attribute nodes.
    ///
    /// ```rust
    /// let mut xot = xot::Xot::new();
    /// let root = xot.parse("<p><a/><b><c/><d/><e/></b><f><g/><h/></f></p>").unwrap();
    /// let p = xot.document_element(root).unwrap();
    /// let a = xot.first_child(p).unwrap();
    /// let b = xot.next_sibling(a).unwrap();
    /// let c = xot.first_child(b).unwrap();
    /// let d = xot.next_sibling(c).unwrap();
    /// let e = xot.next_sibling(d).unwrap();
    /// let f = xot.next_sibling(b).unwrap();
    /// let g = xot.first_child(f).unwrap();
    /// let h = xot.next_sibling(g).unwrap();
    /// let siblings = xot.following(c).collect::<Vec<_>>();
    /// assert_eq!(siblings, vec![d, e, f, g, h]);
    /// ```
    pub fn following(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        // start with an empty iterator
        let mut joined_iterator: Box<dyn Iterator<Item = Node>> = Box::new(std::iter::empty());
        let mut current_parent = Some(node);
        while let Some(parent) = current_parent {
            let mut current_sibling = parent;
            while let Some(current) = self.next_sibling(current_sibling) {
                // add descendants of next sibling
                joined_iterator =
                    Box::new(joined_iterator.chain(Box::new(self.descendants(current))));
                current_sibling = current;
            }
            current_parent = self.parent(parent);
        }
        joined_iterator
    }

    /// Preceding nodes in document order
    ///
    /// These are nodes that come before this node in document order,
    /// without that node itself, its ancestors, or its descendants.
    ///
    /// Does not include namespace or attribute nodes.
    ///
    /// ```rust
    /// let mut xot = xot::Xot::new();
    /// let root = xot.parse("<p><a/><b><c/><d/><e/></b><f><g/><h/></f></p>").unwrap();
    /// let p = xot.document_element(root).unwrap();
    /// let a = xot.first_child(p).unwrap();
    /// let b = xot.next_sibling(a).unwrap();
    /// let c = xot.first_child(b).unwrap();
    /// let d = xot.next_sibling(c).unwrap();
    /// let e = xot.next_sibling(d).unwrap();
    /// let f = xot.next_sibling(b).unwrap();
    /// let g = xot.first_child(f).unwrap();
    /// let h = xot.next_sibling(g).unwrap();
    /// let siblings = xot.preceding(e).collect::<Vec<_>>();
    /// assert_eq!(siblings, vec![d, c, a]);
    /// let siblings = xot.preceding(h).collect::<Vec<_>>();
    /// assert_eq!(siblings, vec![g, e, d, c, b, a]);
    /// ```
    pub fn preceding(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        // start with an empty iterator
        let mut joined_iterator: Box<dyn Iterator<Item = Node>> = Box::new(std::iter::empty());
        let mut current_parent = Some(node);
        while let Some(parent) = current_parent {
            let mut current_sibling = parent;
            while let Some(current) = self.previous_sibling(current_sibling) {
                // add descendants of previous sibling, reversed
                // this unfortunately requires an extra allocation, as descendants
                // is not a double iterator.
                let descendants = Box::new(self.descendants(current).collect::<Vec<_>>());
                let reverse_descendants = descendants.into_iter().rev();
                joined_iterator = Box::new(joined_iterator.chain(Box::new(reverse_descendants)));
                current_sibling = current;
            }
            current_parent = self.parent(parent);
        }
        joined_iterator
    }

    /// Traverse over node edges.
    ///
    /// This can be used to traverse the tree in document order iteratively
    /// without the need for recursion, while getting structure information
    /// (unlike [`Xot::descendants`] which doesn't retain structure
    /// information).
    ///
    /// For the tree `<a><b/></a>` this generates a [`NodeEdge::Start`] for
    /// `<a>`, then a [`NodeEdge::Start`] for `<b>`, immediately followed by a
    /// [`NodeEdge::End`] for `<b>`, and finally a [`NodeEdge::End`] for `<a>`.
    ///
    /// For value types other than element or root, the start and end always
    /// come as pairs without any intervening edges.
    ///
    /// This includes edges for namespace and attribute nodes.
    ///
    /// ```rust
    /// let mut xot = xot::Xot::new();
    /// let root = xot.parse("<a><b>Text</b></a>").unwrap();
    /// let a = xot.document_element(root).unwrap();
    /// let b = xot.first_child(a).unwrap();
    /// let text = xot.first_child(b).unwrap();
    /// let edges = xot.traverse(a).collect::<Vec<_>>();
    /// assert_eq!(edges, vec![
    ///  xot::NodeEdge::Start(a),
    ///  xot::NodeEdge::Start(b),
    ///  xot::NodeEdge::Start(text),
    ///  xot::NodeEdge::End(text),
    ///  xot::NodeEdge::End(b),
    ///  xot::NodeEdge::End(a),
    /// ]);
    /// ```
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

    /// Traverse over nodes in level order.
    ///
    /// This is a breath first traversal, where each level is visited in turn.
    /// Sequences of nodes with a different parent are separated by
    /// [`LevelOrder::End`].
    ///
    /// For the tree `<a><b><d/></b><c><e/></c></a>` this generates a
    /// [`LevelOrder::Node`] for `<a>`, then a [`LevelOrder::End`]. Next, a
    /// [`LevelOrder::Node`] for `<b/>` and `</c>` are generated, again
    /// followed by a [`LevelOrder::End`]. Then a [`LevelOrder::Node`] is
    /// generated for `<d/>`, followed by a [`LevelOrder::End`]. Finally a
    /// [`LevelOrder::Node`] is generated for `<e/>`, followed by a
    /// [`LevelOrder::End`].
    ///
    /// This does not include namespace or attribute nodes.
    ///
    /// ```rust
    /// let mut xot = xot::Xot::new();
    /// let root = xot.parse("<a><b><d/></b><c><e/></c></a>").unwrap();
    /// let a = xot.document_element(root).unwrap();
    /// let b = xot.first_child(a).unwrap();
    /// let d = xot.first_child(b).unwrap();
    /// let c = xot.next_sibling(b).unwrap();
    /// let e = xot.first_child(c).unwrap();
    ///
    /// let levels = xot.level_order(a).collect::<Vec<_>>();
    /// assert_eq!(levels, vec![
    ///   xot::LevelOrder::Node(a),
    ///   xot::LevelOrder::End,
    ///   xot::LevelOrder::Node(b),
    ///   xot::LevelOrder::Node(c),
    ///   xot::LevelOrder::End,
    ///   xot::LevelOrder::Node(d),
    ///   xot::LevelOrder::End,
    ///   xot::LevelOrder::Node(e),
    ///   xot::LevelOrder::End,
    /// ]);
    /// ```
    pub fn level_order(&self, node: Node) -> impl Iterator<Item = LevelOrder> + '_ {
        level_order_traverse(self, node)
    }
}
