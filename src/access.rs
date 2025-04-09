use indextree::NodeEdge as IndexTreeNodeEdge;

use crate::error::Error;
use crate::levelorder::{level_order_traverse, LevelOrder};
use crate::nodemap::{category_predicate, Attributes, Namespaces};
use crate::output::NamespaceDeclarations;
use crate::xmlvalue::{Value, ValueCategory, ValueType};
use crate::xotdata::{Node, Xot};
use crate::{NameId, NamespaceId, PrefixId, Prefixes};

/// Traversal axis.
///
/// This can be used with `[Xot::Axis]` to traverse the tree in different ways.
///
/// The axis behaviors are based on the XPath specification.
///
/// Note that the namespace axis is not supported; it's tricky to support as it
/// includes all namespace nodes in scope of an element, not just those
/// namespaces defined on that element, and has not been a requirement since
/// XPath 2.0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Axis {
    /// The children of the node. Equivalent to [`Xot::children`].
    Child,
    /// The descendants of the node, without the current node itself.
    Descendant,
    /// The parent of the node, or an empty iterator.
    Parent,
    /// The ancestors of the node, without the current node itself.
    Ancestor,
    /// The siblings following the node, without the current sibling.
    /// Equivalent to [`Xot::following_siblings`].
    FollowingSibling,
    /// The siblings preceding the node, without the current sibling.
    /// Equivalent to [`Xot::preceding_siblings`].
    PrecedingSibling,
    /// The nodes following the node. Equivalent to [`Xot::following`].
    Following,
    /// The nodes preceding the node. Equivalent to [`Xot::preceding`].
    Preceding,
    /// The attributes nodes of this node. Equivalent to [`Xot::attribute_nodes`].
    Attribute,
    /// The node itself as an iterator.
    Self_,
    /// The node and its descendants, in document order. Equivalent to
    /// [`Xot::descendants`].
    DescendantOrSelf,
    /// The node and its ancestors. Equivalent to [`Xot::ancestors`].
    AncestorOrSelf,
}

/// Node edges.
///
/// Used by [`Xot::traverse`] and [`Xot::reverse_traverse`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeEdge {
    /// The start edge of a node. In case of an element
    /// this is the start tag. In case of document this is
    /// the start of the document.
    Start(Node),
    /// The end edge of a node. In case of an element
    /// this is the end tag. In case of document this is the end
    /// of the document. For any other values, the
    /// end edge occurs immediately after the start
    /// edge.
    End(Node),
}

impl NodeEdge {
    /// Returns the starting or ending node.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut xot = xot::Xot::new();
    /// let root = xot.parse("<p><a/><b><c/></b></p>").unwrap();
    /// let p = xot.document_element(root).unwrap();
    /// let a = xot.first_child(p).unwrap();
    /// let b = xot.next_sibling(a).unwrap();
    /// let c = xot.first_child(b).unwrap();
    /// let traversed: Vec<xot::Node> = xot
    ///     .traverse(p)
    ///     .map(|edge| edge.node())
    ///     .collect();
    /// assert_eq!(traversed, &[p, a, a, b, c, c, b, p]);
    #[must_use]
    pub fn node(self) -> Node {
        match self {
            Self::Start(node) | Self::End(node) => node,
        }
    }

    /// Returns the next edge in depth-first traversal order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut xot = xot::Xot::new();
    /// let root = xot.parse("<p><a/><b><c/><d/><e/></b><f><g/><h/></f></p>").unwrap();
    /// let traversed: Vec<xot::NodeEdge> = xot.traverse(root).collect();
    ///
    /// let mut current = xot::NodeEdge::Start(root);
    /// let mut traversed2 = vec![current];
    /// while let Some(next) = current.next(&xot) {
    ///     if let xot::NodeEdge::End(node) = current {
    ///         // You can use `&mut Xot` inside the loop.
    ///         xot.remove(node);
    ///     }
    ///
    ///     traversed2.push(next);
    ///     current = next;
    /// }
    /// assert_eq!(traversed, traversed2);
    /// ```
    #[must_use]
    pub fn next(self, xot: &Xot) -> Option<Self> {
        match self {
            Self::Start(current) => xot
                .first_child(current)
                .map(Self::Start)
                .or(Some(Self::End(current))),
            Self::End(current) => xot
                .next_sibling(current)
                .map(Self::Start)
                .or_else(|| xot.parent(current).map(Self::End)),
        }
    }

    /// Returns the previous edge in depth-first traversal order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut xot = xot::Xot::new();
    /// let root = xot.parse("<p><a/><b><c/><d/><e/></b><f><g/><h/></f></p>").unwrap();
    /// let rev_traversed: Vec<xot::NodeEdge> = xot.reverse_traverse(root).collect();
    ///
    /// let mut current = xot::NodeEdge::End(root);
    /// let mut rev_traversed2 = vec![current];
    /// while let Some(next) = current.previous(&xot) {
    ///     if let xot::NodeEdge::Start(node) = current {
    ///         // You can use `&mut Xot` inside the loop.
    ///         xot.remove(node);
    ///     }
    ///
    ///     rev_traversed2.push(next);
    ///     current = next;
    /// }
    /// assert_eq!(rev_traversed, rev_traversed2);
    /// ```
    #[must_use]
    pub fn previous(self, xot: &Xot) -> Option<Self> {
        match self {
            Self::End(current) => xot
                .last_child(current)
                .map(Self::End)
                .or(Some(Self::Start(current))),
            Self::Start(current) => xot
                .previous_sibling(current)
                .map(Self::End)
                .or_else(|| xot.parent(current).map(Self::Start)),
        }
    }
}

/// ## Read-only access
///
/// These are functions that provide read-only access to the tree.
impl Xot {
    /// Obtain the document element from the document node.
    ///
    /// Returns [`Error::NotDocument`](`crate::error::Error::NotDocument`)
    /// error if this is not the document node.
    ///
    /// This accepts document nodes representing fragments. If the fragment has
    /// multiple elements, it returns the first element. If the fragment has no
    /// elements, it errors with [`Error::NoElementAtTopLevel`].
    ///
    /// ```rust
    /// let mut xot = xot::Xot::new();
    ///
    /// let doc = xot.parse("<p>Example</p>").unwrap();
    ///
    /// let doc_el = xot.document_element(doc).unwrap();
    ///
    /// // Check that we indeed have the `p` element
    /// let p_name = xot.name("p").unwrap();
    /// assert_eq!(xot.element(doc_el).unwrap().name(), p_name);
    /// ```
    pub fn document_element(&self, node: Node) -> Result<Node, Error> {
        if self.value_type(node) != ValueType::Document {
            return Err(Error::NotDocument(node));
        }
        for child in self.children(node) {
            if let Value::Element(_) = self.value(child) {
                return Ok(child);
            }
        }
        Err(Error::NoElementAtTopLevel)
    }

    /// Given a node indicating a document, check if it is a well-formed
    /// document.
    ///
    /// This checks that the document has a single document element, and that
    /// there are no text nodes as the top level. If it is a a well-formed
    /// document, this produces no errors.
    ///
    /// If this is not supplied with a document node,
    /// this produces a [`Error::NotDocument`] error.
    ///
    /// If there are no elements at the top level, this produces a
    /// [`Error::NoElementAtTopLevel`] error.
    ///
    /// If there are multiple elements at the top level, this produces a
    /// [`Error::MultipleElementsAtTopLevel`] error.
    ///
    /// If there is a text node at the top level, this produces a
    /// [`Error::TextAtTopLevel`] error.
    ///
    /// If there is a illegal content (a namespace node, an attribute node
    /// or a document node) at the top level under a document node,
    /// this produces a [`Error::IllegalAtTopLevel`] error.
    pub fn validate_well_formed_document(&self, node: Node) -> Result<(), Error> {
        if self.value_type(node) != ValueType::Document {
            return Err(Error::NotDocument(node));
        }

        let mut element_count = 0;
        for child in self.children(node) {
            match self.value(child) {
                Value::Element(_) => element_count += 1,
                // no text nodes at the top level
                Value::Text(_) => return Err(Error::TextAtTopLevel(child)),
                Value::Comment(_) | Value::ProcessingInstruction(_) => {
                    // these we can have as many as we like
                }
                Value::Document | Value::Attribute(_) | Value::Namespace(_) => {
                    // these are completely unexpected under any document node
                    return Err(Error::IllegalAtTopLevel(child));
                }
            }
        }
        if element_count == 0 {
            return Err(Error::NoElementAtTopLevel);
        }
        if element_count > 1 {
            return Err(Error::MultipleElementsAtTopLevel);
        }
        Ok(())
    }

    /// Obtain top element, given node anywhere in a tree.
    ///
    /// In an XML document this is the document element.
    /// In a XML document fragment this is the top element, but it may
    /// have siblings.
    /// If it's an unattached tree, it's the top node of that tree
    pub fn top_element(&self, node: Node) -> Node {
        if self.value_type(node) == ValueType::Document {
            return self.document_element(node).unwrap();
        }
        let mut top = node;
        for ancestor in self.ancestors(node) {
            if let Value::Element(_) = self.value(ancestor) {
                top = ancestor;
            }
        }
        // XXX in an unattached tree this may not be an element.
        top
    }

    /// Obtain root of the tree.
    ///
    /// This is the document node if possible (in a document or fragment), but
    /// in an unattached tree this is the root of that tree.
    pub fn root(&self, node: Node) -> Node {
        self.ancestors(node).last().unwrap()
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
    /// Returns [`None`] if this is the document node or if the node is
    /// unattached to a document.
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

    pub(crate) fn abnormal_children(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        node.get()
            .children(&self.arena)
            .take_while(|n| !self.arena[*n].get().is_normal())
            .map(Node::new)
    }

    pub(crate) fn normal_children(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        node.get()
            .children(&self.arena)
            .skip_while(|n| !self.arena[*n].get().is_normal())
            .map(Node::new)
    }

    /// Attributes accessor.
    ///
    /// Returns a map of [`crate::NameId`] to a String reference representing the
    /// attributes on the element.
    ///
    /// Note that if this is called on a non-element node, you get an empty
    /// map.
    ///
    /// ```rust
    /// let mut xot = xot::Xot::new();
    /// let a = xot.add_name("a");
    /// let root = xot.parse(r#"<p a="A">Example</p>"#).unwrap();
    /// let p = xot.document_element(root).unwrap();
    /// let attributes = xot.attributes(p);
    ///
    /// assert_eq!(attributes.get(a), Some(&"A".to_string()));
    /// ```
    pub fn attributes(&self, node: Node) -> Attributes {
        Attributes::new(self, node)
    }

    /// Get the attribute value for a name.
    ///
    /// Note that if this is invoked non a non-element it's always going to
    /// return None
    pub fn get_attribute(&self, node: Node, name: NameId) -> Option<&str> {
        self.attributes(node).get(name).map(String::as_str)
    }

    /// Namespaces accessor.
    ///
    /// Returns a map of [`crate::PrefixId`] to [`crate::NamespaceId`] representing
    /// the namespace declarations on the element.
    ///
    /// Note that if this is called on a non-element node, you get an empty
    /// map.
    ///
    /// ```rust
    /// let mut xot = xot::Xot::new();
    /// let foo_prefix = xot.add_prefix("foo");
    /// let foo_ns = xot.add_namespace("FOO");
    /// let root = xot.parse(r#"<p xmlns:foo="FOO">Example</p>"#).unwrap();
    /// let p = xot.document_element(root).unwrap();
    /// let namespaces = xot.namespaces(p);
    ///
    /// assert_eq!(namespaces.get(foo_prefix), Some(&foo_ns));
    /// ```
    pub fn namespaces(&self, node: Node) -> Namespaces {
        Namespaces::new(self, node)
    }

    /// Get the namespace for a prefix.
    ///
    /// Note that if this is invoked non a non-element it's always going to
    /// return None
    pub fn get_namespace(&self, node: Node, prefix: PrefixId) -> Option<NamespaceId> {
        self.namespaces(node).get(prefix).copied()
    }

    /// Copy the namespace declarations as a prefixes hash table.
    ///
    /// Sometimes it's more convenient to work with a hash table of
    /// prefixes as opposed to the dynamic [`Xot::namespaces`] node map.
    pub fn prefixes(&self, node: Node) -> Prefixes {
        let mut prefixes = Prefixes::new();
        for (prefix, ns) in self.namespaces(node).iter() {
            prefixes.insert(prefix, *ns);
        }
        prefixes
    }

    /// Copy the namespace declarations as a namespace declarations vec.
    pub fn namespace_declarations(&self, node: Node) -> NamespaceDeclarations {
        self.namespaces(node)
            .iter()
            .map(|(prefix, ns)| (prefix, *ns))
            .collect()
    }

    pub(crate) fn has_namespace_declarations(&self, node: Node) -> bool {
        self.namespaces(node).iter().next().is_some()
    }

    /// Access the attribute nodes directly.
    pub fn attribute_nodes(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        self.all_children(node)
            .skip_while(category_predicate(self, ValueCategory::Namespace))
            .take_while(category_predicate(self, ValueCategory::Attribute))
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

    pub(crate) fn internal_first_child(&self, node: Node) -> Option<Node> {
        Some(Node::new(self.arena[node.get()].first_child()?))
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

    pub(crate) fn internal_last_child(&self, node: Node) -> Option<Node> {
        Some(Node::new(self.arena[node.get()].last_child()?))
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

    pub(crate) fn internal_next_sibling(&self, node: Node) -> Option<Node> {
        Some(Node::new(self.arena[node.get()].next_sibling()?))
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

    pub(crate) fn internal_previous_sibling(&self, node: Node) -> Option<Node> {
        Some(Node::new(self.arena[node.get()].previous_sibling()?))
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
            .children(self.arena())
            .rev()
            .take_while(|n| self.arena[*n].get().is_normal())
            .map(Node::new)
    }

    fn normal_filter(&self) -> impl Fn(&indextree::NodeId) -> bool + '_ {
        |node_id| self.arena[*node_id].get().is_normal()
    }

    fn normal_node_filter(&self) -> impl Fn(Node) -> bool + '_ {
        move |node| self.arena[node.get()].get().is_normal()
    }

    fn normal_edge_filter(&self) -> impl Fn(&indextree::NodeEdge) -> bool + '_ {
        move |edge| {
            let node_id = match edge {
                indextree::NodeEdge::Start(node_id) => node_id,
                indextree::NodeEdge::End(node_id) => node_id,
            };
            self.arena[*node_id].get().is_normal()
        }
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

    /// All the descendants of this node.
    ///
    /// This includes this one, and namespace and attribute nodes,
    /// all in document order, where namespace nodes come before
    /// attribute nodes and attribute nodes come before normal children
    pub fn all_descendants(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        node.get().descendants(self.arena()).map(Node::new)
    }

    /// Reverse preorder traversal from node.
    ///
    /// This starts with the given node, and then back to the root of the tree,
    /// in reverse preorder (document order) traversal. This means all nodes
    /// that exist previously in preorder are visited. It can be seen as
    /// the inverse of a descendants traversal.
    ///
    /// Namespace and attribute nodes are not included in the traversal.
    pub fn reverse_preorder(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        ReversePreorder::new(self, node, self.normal_node_filter())
    }

    /// Reverse preorder traversal from node for all nodes.
    ///
    /// Like [`Xot::reverse_preorder`] but includes namespace and
    /// attribute nodes too.
    pub fn all_reverse_preorder(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        ReversePreorder::new(self, node, |_| true)
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
        Following::new(self, self.next_sibling(node), self.normal_node_filter())
    }

    /// Following nodes in document order.
    ///
    /// Like [`Xot::following`] but includes namespace and
    /// attribute nodes too.
    pub fn all_following(&self, node: Node) -> impl Iterator<Item = Node> + '_ {
        Following::new(self, self.internal_next_sibling(node), |_| true)
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
    /// This does not include edges for namespace and attribute nodes.
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
        node.get()
            .traverse(self.arena())
            .filter(self.normal_edge_filter())
            .map(|edge| match edge {
                IndexTreeNodeEdge::Start(node_id) => NodeEdge::Start(Node::new(node_id)),
                IndexTreeNodeEdge::End(node_id) => NodeEdge::End(Node::new(node_id)),
            })
    }

    /// Traverse nodes, including namespace and attribute nodes.
    pub fn all_traverse(&self, node: Node) -> impl Iterator<Item = NodeEdge> + '_ {
        node.get().traverse(self.arena()).map(|edge| match edge {
            IndexTreeNodeEdge::Start(node_id) => NodeEdge::Start(Node::new(node_id)),
            IndexTreeNodeEdge::End(node_id) => NodeEdge::End(Node::new(node_id)),
        })
    }

    /// Traverse over node edges in reverse order.
    ///
    /// Like [`Xot::traverse`] but in reverse order.
    ///
    /// Note that you still have to start with a top-level node, it just traverses through
    /// its descendants in a reverse order.
    pub fn reverse_traverse(&self, node: Node) -> impl Iterator<Item = NodeEdge> + '_ {
        node.get()
            .reverse_traverse(self.arena())
            .filter(self.normal_edge_filter())
            .map(|edge| match edge {
                IndexTreeNodeEdge::Start(node_id) => NodeEdge::Start(Node::new(node_id)),
                IndexTreeNodeEdge::End(node_id) => NodeEdge::End(Node::new(node_id)),
            })
    }

    /// Traverse nodes, including namespace and attribute nodes, in reverse order.
    ///
    /// This is like [`Xot::all_traverse`] but includes namespace and
    pub fn reverse_all_traverse(&self, node: Node) -> impl Iterator<Item = NodeEdge> + '_ {
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

    /// Axis-based traversal.
    ///
    /// Use an [`crate::Axis`] to traverse the tree in a way defined by
    /// XPath.
    ///
    /// `<https://developer.mozilla.org/en-US/docs/Web/XPath/Axes>`
    pub fn axis(&self, axis: Axis, node: Node) -> Box<dyn Iterator<Item = Node> + '_> {
        use Axis::*;
        match axis {
            Child => Box::new(self.children(node)),
            Descendant => {
                let mut descendants = self.descendants(node);
                // since this includes self we get rid of it here
                descendants.next();
                Box::new(descendants)
            }
            Parent => {
                if let Some(parent) = self.parent(node) {
                    Box::new(std::iter::once(parent))
                } else {
                    Box::new(std::iter::empty())
                }
            }
            Ancestor => {
                let parent = self.parent(node);
                if let Some(parent) = parent {
                    // the ancestors of the parent include self, which is
                    // what we want as the parent is already taken
                    // We can't get a Node::Attribute or Node::Namespace
                    // because we just took the parent
                    Box::new(self.ancestors(parent))
                } else {
                    Box::new(std::iter::empty())
                }
            }
            FollowingSibling => {
                let mut following = self.following_siblings(node);
                // consume the self sibling
                following.next();
                Box::new(following)
            }
            PrecedingSibling => {
                let mut preceding = self.preceding_siblings(node);
                // consume the self sibling
                preceding.next();
                Box::new(preceding)
            }
            Following => Box::new(self.following(node)),
            Preceding => Box::new(self.preceding(node)),
            Axis::Self_ => Box::new(std::iter::once(node)),
            DescendantOrSelf => Box::new(self.descendants(node)),
            AncestorOrSelf => Box::new(self.ancestors(node)),
            Attribute => Box::new(self.attribute_nodes(node)),
        }
    }

    /// Get (element) node in a document node that has an xml:id attribute of
    /// given value.
    ///
    /// If that id does not exist, returns [`None`].
    pub fn xml_id_node(&self, document_node: Node, value: &str) -> Option<Node> {
        let value_nodes = self.id_nodes_map.get(&document_node.get())?;
        value_nodes.get(value).map(|node_id| Node::new(*node_id))
    }
}

struct ReversePreorder<'a, F: Fn(Node) -> bool> {
    xot: &'a Xot,
    current: Option<Node>,
    filter: F,
}

impl<'a, F: Fn(Node) -> bool> ReversePreorder<'a, F> {
    fn new(xot: &'a Xot, current: Node, filter: F) -> Self {
        Self {
            xot,
            current: Some(current),
            filter,
        }
    }
}

impl<F: Fn(Node) -> bool> Iterator for ReversePreorder<'_, F> {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.current {
            // if we have a previous sibling, go to the rightmost deepest descendant
            let previous = self.xot.internal_previous_sibling(current);
            let next = if let Some(mut node) = previous {
                while let Some(last_child) = self.xot.internal_last_child(node) {
                    node = last_child;
                }
                Some(node)
            } else {
                // if we don't have a previous sibling, go to parent
                self.xot.parent(current)
            };
            self.current = next;
            // if we don't pass the filter, go to the next node
            if !(self.filter)(current) {
                return self.next();
            }
            Some(current)
        } else {
            None
        }
    }
}

struct Following<'a, F> {
    xot: &'a Xot,
    current: Option<Node>,
    filter: F,
}

impl<'a, F: Fn(Node) -> bool> Following<'a, F> {
    fn new(xot: &'a Xot, current: Option<Node>, filter: F) -> Self {
        Self {
            xot,
            current,
            filter,
        }
    }

    fn following(node: Node, xot: &Xot) -> Option<Node> {
        if let Some(next_sibling) = xot.internal_next_sibling(node) {
            Some(next_sibling)
        } else {
            let mut current = node;
            while let Some(parent) = xot.parent(current) {
                let sibling = xot.internal_next_sibling(parent);
                if let Some(sibling) = sibling {
                    return Some(sibling);
                } else {
                    current = parent;
                }
            }
            None
        }
    }
}

impl<F: Fn(Node) -> bool> Iterator for Following<'_, F> {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.current?;
        self.current = if let Some(first_child) = self.xot.internal_first_child(node) {
            Some(first_child)
        } else {
            Self::following(node, self.xot)
        };
        // if we don't pass the filter, then we do the next one
        if !(self.filter)(node) {
            return self.next();
        }
        Some(node)
    }
}
