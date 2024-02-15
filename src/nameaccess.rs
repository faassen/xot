use ahash::{HashMap, HashSet};
use genawaiter::rc::gen;
use genawaiter::yield_;

use crate::access::NodeEdge;
use crate::error::Error;
use crate::fullname::{Fullname, FullnameSerializer};
use crate::id::{Name, NameId, NamespaceId, PrefixId};
use crate::xmlvalue::Prefixes;
use crate::xotdata::{Node, Xot};
use crate::Value;

/// ## Names, namespaces and prefixes.
///
/// Xot does not let you use names, prefixes and URIs directly. Instead you use
/// the types [`NameId`], [`NamespaceId`] and [`PrefixId`] to
/// refer to these.
///
/// This has some advantages:
///
/// * It's faster to compare and hash names, namespaces and prefixes.
///
/// * It takes less memory to store a tree.
///
/// * You get type-checks and can't mix up names, namespaces and prefixes.
///
/// Names, namespaces and prefixes are shared in a single Xot, so are the same
/// in multiple trees. This makes it safe to copy and move nodes between trees.
/// If you care about the readability of the serialized XML you do need to
/// ensure that each tree uses `xmlns` attributes to declare the namespaces it
/// uses; otherwise prefixes are generated during serialization.
///
/// The minor drawback is that you need to use multiple steps to create a name,
/// prefix or namespace for use, or to access the string value of a name,
/// prefix or namepace. This drawback may be an advantage at times, as typical
/// code needs to use a single name, namespace or prefix multiple times, so
/// assigning to a variable is more convenient than repeating strings.
impl Xot {
    /// Look up name without a namespace.
    ///
    /// This is the immutable version of [`Xot::add_name`]; it returns
    /// `None` if the name doesn't exist.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// assert!(xot.name("a").is_none());
    ///
    /// let name = xot.add_name("a");
    /// assert_eq!(xot.name("a"), Some(name));
    /// ```
    pub fn name(&self, name: &str) -> Option<NameId> {
        self.name_ns(name, self.no_namespace_id)
    }

    /// Add name without a namespace.
    ///
    /// If the name already exists, return its id, otherwise creates it.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    ///
    /// let name = xot.add_name("a");
    /// // the namespace is "" for no namespace
    /// assert_eq!(xot.name_ns_str(name), ("a", ""));
    ///
    /// let root = xot.parse(r#"<doc/>"#)?;
    /// let doc_el = xot.document_element(root).unwrap();
    /// // add an element, using the name
    /// let node = xot.append_element(doc_el, name)?;
    ///
    /// assert_eq!(xot.to_string(root)?, "<doc><a/></doc>");
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn add_name(&mut self, name: &str) -> NameId {
        self.add_name_ns(name, self.no_namespace_id)
    }

    /// Look up name with a namespace.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    ///
    /// let ns = xot.add_namespace("http://example.com");
    /// let name = xot.add_name_ns("a", ns);
    /// assert_eq!(xot.name_ns_str(name), ("a", "http://example.com"));
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    ///
    /// Look up name of an element:
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse(r#"<doc xmlns="http://example.com"><a/></doc>"#)?;
    /// let doc_el = xot.document_element(root).unwrap();
    ///
    /// let doc_value = xot.element(doc_el).unwrap();
    ///
    /// // get the name of the element
    /// let name = xot.name_ns_str(doc_value.name());
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn name_ns(&self, name: &str, namespace_id: NamespaceId) -> Option<NameId> {
        self.name_lookup.get_id(&Name::new(name, namespace_id))
    }

    /// Add name with a namespace.
    ///
    /// If the name already exists, return its id.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    ///
    /// let ns = xot.add_namespace("http://example.com");
    /// let name_a = xot.add_name_ns("a", ns);
    ///
    /// let root = xot.parse(r#"<doc xmlns="http://example.com"><a/></doc>"#)?;
    /// let doc_el = xot.document_element(root).unwrap();
    /// let a_el = xot.first_child(doc_el).unwrap();
    ///
    /// let doc_value = xot.element(doc_el).unwrap();
    /// let a_value = xot.element(a_el).unwrap();
    ///
    /// // we know a is the right name, but doc is not
    /// assert_eq!(a_value.name(), name_a);
    /// assert_ne!(doc_value.name(), name_a);
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn add_name_ns(&mut self, name: &str, namespace_id: NamespaceId) -> NameId {
        self.name_lookup.get_id_mut(&Name::new(name, namespace_id))
    }

    /// Look up namespace.
    ///
    /// This is the immutable version of [`Xot::add_namespace`]; it returns
    /// `None` if the namespace doesn't exist.
    pub fn namespace(&self, namespace: &str) -> Option<NamespaceId> {
        self.namespace_lookup.get_id(namespace)
    }

    /// Add namespace.
    ///
    /// If the namespace already exists, return its id.
    pub fn add_namespace(&mut self, namespace: &str) -> NamespaceId {
        self.namespace_lookup.get_id_mut(namespace)
    }

    /// Look up prefix.
    ///
    /// This is the immutable version of [`Xot::add_prefix`]; it returns
    /// `None` if the prefix doesn't exist.
    pub fn prefix(&self, prefix: &str) -> Option<PrefixId> {
        self.prefix_lookup.get_id(prefix)
    }

    /// Add prefix.
    ///
    /// If the prefix already exists, return its id.
    pub fn add_prefix(&mut self, prefix: &str) -> PrefixId {
        self.prefix_lookup.get_id_mut(prefix)
    }

    /// No namespace
    ///
    /// Returns the namespace id used when an element or attribute
    /// isn't in any namespace.
    #[inline]
    pub fn no_namespace(&self) -> NamespaceId {
        self.no_namespace_id
    }

    /// Empty prefix
    ///
    /// Returns the prefix id used when an element or attribute
    /// doesn't have a prefix.
    #[inline]
    pub fn empty_prefix(&self) -> PrefixId {
        self.empty_prefix_id
    }

    /// XML prefix
    ///
    /// The prefix `xml` used for the XML namespace.
    #[inline]
    pub fn xml_prefix(&self) -> PrefixId {
        self.xml_prefix_id
    }

    /// XML namespace
    ///
    /// Returns the namespace id used for the XML namespace.
    ///
    /// Also known as `http://wwww.w3.org/XML/1998/namespace`
    #[inline]
    pub fn xml_namespace(&self) -> NamespaceId {
        self.xml_namespace_id
    }

    /// xml:space
    ///
    /// Returns the name id used for the `xml:space` attribute.
    #[inline]
    pub fn xml_space_name(&self) -> NameId {
        self.xml_space_id
    }

    /// Look up localname, namespace uri for name id
    ///
    /// If this name id is not in a namespace, the namespace uri is the
    /// empty string.
    ///
    /// No namespace:
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse(r#"<doc><a/></doc>"#)?;
    /// let doc_el = xot.document_element(root).unwrap();
    /// let a_el = xot.first_child(doc_el).unwrap();
    ///
    /// let a_value = xot.element(a_el).unwrap();
    ///
    /// let (localname, namespace) = xot.name_ns_str(a_value.name());
    /// assert_eq!(localname, "a");
    /// assert_eq!(namespace, "");
    /// # Ok::<(), xot::Error>(())
    /// ```
    ///
    /// With namespace:
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse(r#"<doc xmlns="http://example.com"><a/></doc>"#)?;
    /// let doc_el = xot.document_element(root).unwrap();
    /// let a_el = xot.first_child(doc_el).unwrap();
    ///
    /// let a_value = xot.element(a_el).unwrap();
    ///
    /// let (localname, namespace) = xot.name_ns_str(a_value.name());
    /// assert_eq!(localname, "a");
    /// assert_eq!(namespace, "http://example.com");
    /// # Ok::<(), xot::Error>(())
    /// ```
    #[inline]
    pub fn name_ns_str(&self, name: NameId) -> (&str, &str) {
        let name = self.name_lookup.get_value(name);
        let namespace = self.namespace_lookup.get_value(name.namespace_id);
        (name.name.as_ref(), namespace)
    }

    /// Get the localname of a name.
    #[inline]
    pub fn local_name_str(&self, name: NameId) -> &str {
        let name = self.name_lookup.get_value(name);
        name.name.as_ref()
    }

    /// Get the namespace URI of a name
    #[inline]
    pub fn uri_str(&self, name: NameId) -> &str {
        let name = self.name_lookup.get_value(name);
        self.namespace_str(name.namespace_id)
    }

    /// Look up namespace uri for namespace id
    ///
    /// An empty string slice indicates the no namespace.
    #[inline]
    pub fn namespace_str(&self, namespace: NamespaceId) -> &str {
        let namespace = self.namespace_lookup.get_value(namespace);
        namespace
    }

    /// Look up string slice for prefix id
    ///
    /// If the prefix id is the empty prefix, the string slice is the empty string.
    #[inline]
    pub fn prefix_str(&self, prefix: PrefixId) -> &str {
        let prefix = self.prefix_lookup.get_value(prefix);
        prefix
    }

    /// Get the Namespace for a Name
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let ns = xot.add_namespace("http://example.com");
    /// let name = xot.add_name_ns("a", ns);
    ///
    /// assert_eq!(xot.namespace_for_name(name), ns);
    /// # Ok::<(), xot::Error>(())
    /// ```
    #[inline]
    pub fn namespace_for_name(&self, name: NameId) -> NamespaceId {
        self.name_lookup.get_value(name).namespace_id
    }

    /// Full name.
    ///
    /// Given a context node, determine the full name string of the given name.
    ///
    /// If the name doesn't have a namespace, that's identical to the localname.
    /// If the name is in a namespace, a prefix is looked up. If no prefix
    /// exists, that's an error.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// // prefixed
    /// let mut xot = Xot::new();
    /// let doc = xot.parse(r#"<foo:doc xmlns:foo="http://example.com"/>"#)?;
    /// let doc_el = xot.document_element(doc).unwrap();
    /// let name = xot.node_name(doc_el).unwrap();
    /// let full_name = xot.full_name(doc_el, name)?;
    /// assert_eq!(full_name, "foo:doc");
    ///
    /// // default namespace
    /// let doc = xot.parse(r#"<doc xmlns="http://example.com"/>"#)?;
    /// let doc_el = xot.document_element(doc).unwrap();
    /// let name = xot.node_name(doc_el).unwrap();
    /// let full_name = xot.full_name(doc_el, name)?;
    /// assert_eq!(full_name, "doc");
    ///
    /// // no namespace
    /// let doc = xot.parse(r#"<doc/>"#)?;
    /// let doc_el = xot.document_element(doc).unwrap();
    /// let name = xot.node_name(doc_el).unwrap();
    /// let full_name = xot.full_name(doc_el, name)?;
    /// assert_eq!(full_name, "doc");
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn full_name(&self, node: Node, name: NameId) -> Result<String, Error> {
        let namespace = self.namespace_for_name(name);
        let local_name = self.local_name_str(name);
        if namespace == self.no_namespace() {
            return Ok(local_name.to_string());
        }
        // look up the prefix for the namespace
        if let Some(prefix) = self.prefix_for_namespace(node, namespace) {
            let prefix = self.prefix_str(prefix);
            if !prefix.is_empty() {
                Ok(format!("{}:{}", prefix, local_name))
            } else {
                Ok(local_name.to_string())
            }
        } else {
            Err(Error::MissingPrefix(namespace))
        }
    }

    /// Given a node, give back the name id of this node.
    ///
    /// For elements and attribute that is their name, for processing
    /// instructions this is a name based on the target attribute.
    ///
    /// For anything else, it's `None`.
    pub fn node_name(&self, node: Node) -> Option<NameId> {
        match self.value(node) {
            Value::Element(element) => Some(element.name()),
            Value::Text(..) => None,
            Value::ProcessingInstruction(pi) => Some(pi.target()),
            Value::Comment(..) => None,
            Value::Document => None,
            Value::Attribute(attribute) => Some(attribute.name()),
            Value::Namespace(_) => None,
        }
    }

    /// Check whether a prefix is defined in node or its ancestors.
    pub fn is_prefix_defined(&self, node: Node, prefix: PrefixId) -> bool {
        for ancestor in self.ancestors(node) {
            if self.namespaces(ancestor).contains_key(prefix) {
                return true;
            }
        }
        if self.base_prefixes().contains_key(&prefix) {
            return true;
        }
        false
    }

    /// Find prefixes we inherit from ancestors and aren't defined locally
    pub fn inherited_prefixes(&self, node: Node) -> Prefixes {
        let prefixes = if let Some(node) = self.parent(node) {
            self.prefixes_in_scope(node)
        } else {
            Prefixes::new()
        };
        // now filter these by namespaces actually required
        let unresolved_namespaces = HashSet::from_iter(self.unresolved_namespaces(node));
        prefixes
            .into_iter()
            .filter(|(_, ns)| unresolved_namespaces.contains(ns))
            .collect::<Prefixes>()
    }

    /// Find prefix for a namespace in node or ancestors.
    ///
    /// Returns `None` if no prefix is defined for the namespace.
    pub fn prefix_for_namespace(&self, node: Node, namespace: NamespaceId) -> Option<PrefixId> {
        for ancestor in self.ancestors(node) {
            for (key, value) in self.namespaces(ancestor).iter() {
                if *value == namespace {
                    return Some(key);
                }
            }
        }
        for (key, value) in self.base_prefixes() {
            if value == namespace {
                return Some(key);
            }
        }
        None
    }

    /// Find namespace for prefix in node or ancestors.
    ///
    /// Return `None` if no namespace is defined for the prefix.
    pub fn namespace_for_prefix(&self, node: Node, prefix: PrefixId) -> Option<NamespaceId> {
        for ancestor in self.ancestors(node) {
            if let Some(namespace) = self.namespaces(ancestor).get(prefix) {
                return Some(*namespace);
            }
        }
        for (key, value) in self.base_prefixes() {
            if key == prefix {
                return Some(value);
            }
        }
        None
    }

    /// Creating missing prefixes.
    ///
    /// Due to creation or moving subtrees you can end up with XML elements or
    /// attributes that have names in a namespace without a prefix to define
    /// the namespace in its ancestors.
    ///
    /// This function creates the missing prefixes on the given node. The
    /// prefixes are named "n0", "n1", "n2", etc.
    ///
    /// You can use this function just before serializing the tree to XML
    /// using [`Xot::write`] or [`Xot::to_string`].
    pub fn create_missing_prefixes(&mut self, node: Node) -> Result<(), Error> {
        let node = if self.is_document(node) {
            self.document_element(node).unwrap()
        } else {
            node
        };
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
                        fullname_serializer.push(&self.prefixes(node));
                        let element_fullname = fullname_serializer.fullname(element.name_id);
                        if let Fullname::MissingPrefix(namespace_id) = element_fullname {
                            missing_namespace_ids.insert(namespace_id);
                        }
                        for name_id in self.attributes(node).keys() {
                            let attribute_fullname = fullname_serializer.fullname_attr(name_id);
                            if let Fullname::MissingPrefix(namespace_id) = attribute_fullname {
                                missing_namespace_ids.insert(namespace_id);
                            }
                        }
                    }
                }
                NodeEdge::End(node) => {
                    if self.is_element(node) {
                        fullname_serializer.pop();
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
        let mut namespaces = self.namespaces_mut(node);

        for (prefix_id, namespace_id) in prefixes_to_add {
            namespaces.insert(prefix_id, *namespace_id);
        }
        Ok(())
    }

    /// Deduplicate namespaces.
    ///
    /// Any namespace definition lower down that defines a prefix for a
    /// namespace that is already known in an ancestor is removed.
    ///
    /// There is a special rule for attributes, as they can only be in a
    /// namespace if they have an explicit prefix; the prefix is not removed if
    /// it overlaps with a default namespace.
    ///
    /// With default namespaces:
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse(r#"<doc xmlns="http://example.com"><a xmlns="http://example.com"/></doc>"#)?;
    /// xot.deduplicate_namespaces(root);
    ///
    /// assert_eq!(xot.to_string(root)?, r#"<doc xmlns="http://example.com"><a/></doc>"#);
    /// # Ok::<(), xot::Error>(())
    /// ```
    ///
    /// With explicit prefixes:
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse(r#"<ns:doc xmlns:ns="http://example.com"><ns:a xmlns:ns="http://example.com"/></ns:doc>"#)?;
    ///
    /// xot.deduplicate_namespaces(root);
    ///
    /// assert_eq!(xot.to_string(root)?, r#"<ns:doc xmlns:ns="http://example.com"><ns:a/></ns:doc>"#);
    /// # Ok::<(), xot::Error>(())
    /// ```
    ///
    /// This also works if you use different prefixes for the same namespace
    /// URI:
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse(r#"<ns:doc xmlns:ns="http://example.com"><other:a xmlns:other="http://example.com"/></ns:doc>"#)?;
    ///
    /// xot.deduplicate_namespaces(root);
    ///
    /// assert_eq!(xot.to_string(root)?, r#"<ns:doc xmlns:ns="http://example.com"><ns:a/></ns:doc>"#);
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn deduplicate_namespaces(&mut self, node: Node) {
        let mut fullname_serializer = FullnameSerializer::new(self);
        let mut fixup_nodes = Vec::new();
        let mut deduplicate_tracker = DeduplicateTracker::new();
        // determine nodes we need to fix up
        for edge in self.traverse(node) {
            match edge {
                NodeEdge::Start(node) => {
                    if self.is_element(node) {
                        // an attribute in a namespace *has* to have a non-empty
                        // prefix. This means we cannot remove a prefix if that
                        // prefix overlaps with a previously defined default
                        // namespace: that's fine for elements which fall
                        // in the default namespace, but not for attributes.
                        // The tracker keeps track of all this.
                        deduplicate_tracker.push(self, node);
                        // we don't need to remove the fixed up prefixes because
                        // as duplicates they will definitely exist.
                        // In fact if we remove them first the push will fail to create
                        // a new entry in the namespace stack, as prefixes can become empty
                        fullname_serializer.push(&self.prefixes(node));
                    }
                }
                NodeEdge::End(node) => {
                    if self.is_element(node) {
                        // to_prefix is only used to determine whether to pop
                        // so should be okay to send here
                        fullname_serializer.pop();
                        deduplicate_tracker.pop();
                        // if we already know a namespace, remove it
                        // we do this at the end so the deduplicate tracker
                        // has had a change to do its work for sub-elements
                        let namespaces = self.namespaces(node);
                        let to_remove = namespaces
                            .iter()
                            .filter_map(|(_, namespace_id)| {
                                if fullname_serializer.is_namespace_known(*namespace_id)
                                    && deduplicate_tracker.is_safe_to_remove(*namespace_id)
                                {
                                    Some(*namespace_id)
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();
                        if !to_remove.is_empty() {
                            fixup_nodes.push((node, to_remove.clone()));
                        }
                    }
                }
            }
        }
        // now actually fix up the nodes, removing superfluous namespaces
        // TODO: this whole thing is a bit a multi-step mess. Perhaps
        // direct namespace node access would help.
        let mut fixup_prefixes = Vec::new();
        for (node, to_remove) in fixup_nodes {
            let namespaces = self.namespaces(node);
            for namespace_id in to_remove {
                let prefixes_to_remove = namespaces
                    .iter()
                    .filter(|(_, ns)| **ns == namespace_id)
                    .map(|(prefix, _)| prefix);
                fixup_prefixes.push((node, prefixes_to_remove.collect::<Vec<_>>()));
            }
        }
        for (node, prefix) in fixup_prefixes {
            let mut namespaces = self.namespaces_mut(node);
            for prefix in prefix {
                namespaces.remove(prefix);
            }
        }
    }

    pub(crate) fn prefixes_in_scope(&self, node: Node) -> Prefixes {
        let mut prefixes = Prefixes::new();
        for ancestor in self.ancestors(node) {
            let namespaces = self.namespaces(ancestor);
            for (prefix_id, namespace_id) in namespaces.iter() {
                // prefixes defined later override those defined earlier
                if prefixes.contains_key(&prefix_id) {
                    continue;
                }
                prefixes.insert(prefix_id, *namespace_id);
            }
        }
        prefixes
    }

    /// Get namespaces without prefix within node or its descendants.
    ///
    /// Any elements or attribute with namespaces that don't have a prefix
    /// defined for them in the context of the node are reported.
    pub fn unresolved_namespaces(&self, node: Node) -> Vec<NamespaceId> {
        let mut namespaces = Vec::new();
        let mut fullname_serializer = FullnameSerializer::new(self);
        for edge in self.traverse(node) {
            match edge {
                NodeEdge::Start(node) => {
                    let element = self.element(node);
                    if let Some(element) = element {
                        fullname_serializer.push(&self.prefixes(node));
                        let namespace_id = self.namespace_for_name(element.name());
                        if !fullname_serializer.is_namespace_known(namespace_id) {
                            namespaces.push(namespace_id);
                        }
                        for name in self.attributes(node).keys() {
                            let namespace_id = self.namespace_for_name(name);
                            if !fullname_serializer.is_namespace_known(namespace_id) {
                                namespaces.push(namespace_id);
                            }
                        }
                    }
                }
                NodeEdge::End(node) => {
                    if self.is_element(node) {
                        fullname_serializer.pop();
                    }
                }
            }
        }
        namespaces
    }

    /// Returns an iterator that yields all the prefix/namespace combinations.
    ///
    /// Once a prefix has been yielded, it's not yielded again, as the
    /// overriding prefix has already been yielded.
    pub fn namespaces_in_scope(
        &self,
        node: Node,
    ) -> impl Iterator<Item = (PrefixId, NamespaceId)> + '_ {
        namespace_traverse(self, node)
    }

    pub(crate) fn base_prefixes(&self) -> Prefixes {
        let mut prefixes = Prefixes::new();
        prefixes.insert(self.xml_prefix_id, self.xml_namespace_id);
        prefixes
    }
}

struct DeduplicateTracker {
    stack: Vec<DeduplicateTrackerEntry>,
}

struct DeduplicateTrackerEntry {
    default_namespace: Option<NamespaceId>,
    in_use_by_attribute: bool,
}

impl DeduplicateTracker {
    fn new() -> Self {
        Self { stack: Vec::new() }
    }

    fn push(&mut self, xot: &Xot, node: Node) {
        let namespaces = xot.namespaces(node);
        let default_namespace = namespaces.get(xot.empty_prefix());
        self.stack.push(DeduplicateTrackerEntry {
            default_namespace: default_namespace.copied(),
            in_use_by_attribute: false,
        });
        for attribute_name in xot.attributes(node).keys() {
            self.attribute_name(xot, attribute_name);
        }
    }

    fn pop(&mut self) {
        self.stack.pop();
    }

    fn attribute_name(&mut self, xot: &Xot, name: NameId) {
        let namespace = xot.namespace_for_name(name);
        for entry in self.stack.iter_mut().rev() {
            if entry.default_namespace == Some(namespace) {
                entry.in_use_by_attribute = true;
                return;
            }
        }
    }

    fn is_safe_to_remove(&self, namespace: NamespaceId) -> bool {
        for entry in self.stack.iter().rev() {
            if entry.default_namespace == Some(namespace) {
                return !entry.in_use_by_attribute;
            }
        }
        true
    }
}

pub(crate) fn namespace_traverse(
    xot: &Xot,
    node: Node,
) -> impl Iterator<Item = (PrefixId, NamespaceId)> + '_ {
    gen!({
        let mut seen: Vec<PrefixId> = Vec::new();
        for ancestor in xot.ancestors(node) {
            let namespaces = xot.namespaces(ancestor);
            for (prefix_id, namespace_id) in namespaces.iter() {
                if seen.contains(&prefix_id) {
                    continue;
                }
                seen.push(prefix_id);
                yield_!((prefix_id, *namespace_id));
            }
        }
        for (prefix_id, namespace_id) in xot.base_prefixes() {
            if seen.contains(&prefix_id) {
                continue;
            }
            seen.push(prefix_id);
            yield_!((prefix_id, namespace_id));
        }
    })
    .into_iter()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefixes_in_scope() {
        let mut xot = Xot::new();
        let root = xot
            .parse(r#"<doc xmlns:foo="http://example.com"><a><b xmlns:foo="http://example.com/foo" xmlns:bar="http://example.com/bar" /></a></doc>"#)
            .unwrap();
        let doc_el = xot.document_element(root).unwrap();
        let a = xot.first_child(doc_el).unwrap();
        let b = xot.first_child(a).unwrap();

        let foo = xot.prefix("foo").unwrap();
        let ns = xot.namespace("http://example.com").unwrap();
        let ns_foo = xot.namespace("http://example.com/foo").unwrap();
        let ns_bar = xot.namespace("http://example.com/bar").unwrap();
        let bar = xot.prefix("bar").unwrap();

        assert_eq!(
            xot.prefixes_in_scope(doc_el),
            Prefixes::from_iter(vec![(foo, ns)])
        );

        assert_eq!(
            xot.prefixes_in_scope(a),
            Prefixes::from_iter(vec![(foo, ns)])
        );

        assert_eq!(
            xot.prefixes_in_scope(b),
            Prefixes::from_iter(vec![(foo, ns_foo), (bar, ns_bar)])
        );
    }
}
