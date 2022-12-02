use indextree::{Arena, NodeId};
use xmlparser::{ElementEnd, Token, Tokenizer};

use crate::document::{namespace_by_prefix, Document};
use crate::error::Error;
use crate::name::{Name, NameLookup};
use crate::namespace::{Namespace, NamespaceId, NamespaceLookup};
use crate::prefix::{Prefix, PrefixId, PrefixLookup};
use crate::xmlnode::{Attributes, Element, NamespaceInfo, XmlNode};

struct ElementBuilder<'a> {
    prefix: &'a str,
    name: &'a str,
    namespace_info: NamespaceInfo,
    // XXX this can't be attributes but has to be map of prefix, name
    // to value that can later be converted once namespaces are known
    attributes: Attributes<'a>,
}

impl<'a> ElementBuilder<'a> {
    fn new(prefix: &'a str, name: &'a str) -> Self {
        ElementBuilder {
            prefix,
            name,
            namespace_info: NamespaceInfo::new(),
            attributes: Attributes::new(),
        }
    }

    fn into_element(
        self,
        document_builder: &mut DocumentBuilder<'a>,
        arena: &mut Arena<XmlNode<'a>>,
    ) -> Result<Element<'a>, Error> {
        let prefix_id = document_builder
            .prefix_lookup
            .get_id(Prefix::new(self.prefix));
        let namespace_id = self
            .namespace_info
            .to_namespace
            .get(&prefix_id)
            .copied()
            .or_else(|| document_builder.namespace_by_prefix(prefix_id, arena));
        let namespace_id =
            namespace_id.ok_or_else(|| Error::UnknownPrefix(self.prefix.to_owned()))?;
        let name = Name::new(self.name, namespace_id);
        let name_id = document_builder.name_lookup.get_id(name);
        Ok(Element {
            name_id,
            namespace_info: self.namespace_info,
            attributes: self.attributes,
        })
    }
}

struct DocumentBuilder<'a> {
    namespace_lookup: NamespaceLookup<'a>,
    prefix_lookup: PrefixLookup<'a>,
    no_namespace_id: NamespaceId,
    empty_prefix_id: PrefixId,
    name_lookup: NameLookup<'a>,
    tree: NodeId,
    current_node_id: NodeId,
    element_builder: Option<ElementBuilder<'a>>,
}

impl<'a> DocumentBuilder<'a> {
    fn new(arena: &mut Arena<XmlNode>) -> Self {
        let mut namespace_lookup = NamespaceLookup::new();
        // XXX absence of namespace is defined as the empty namespace,
        // we should forbid its construction otherwise?
        let no_namespace_id = namespace_lookup.get_id(Namespace::new(""));

        let mut prefix_lookup = PrefixLookup::new();
        let empty_prefix_id = prefix_lookup.get_id(Prefix::new(""));

        let root = arena.new_node(XmlNode::Root);
        DocumentBuilder {
            namespace_lookup,
            prefix_lookup,
            no_namespace_id,
            empty_prefix_id,
            name_lookup: NameLookup::new(),
            tree: root,
            current_node_id: root,
            element_builder: None,
        }
    }

    fn into_document(self) -> Document<'a> {
        Document {
            namespace_lookup: self.namespace_lookup,
            prefix_lookup: self.prefix_lookup,
            name_lookup: self.name_lookup,
            tree: self.tree,
            no_namespace_id: self.no_namespace_id,
        }
    }

    fn element(&mut self, prefix: &'a str, name: &'a str) {
        self.element_builder = Some(ElementBuilder::new(prefix, name));
    }

    fn namespace_by_prefix(
        &self,
        prefix_id: PrefixId,
        arena: &Arena<XmlNode>,
    ) -> Option<NamespaceId> {
        namespace_by_prefix(self.current_node_id, prefix_id, arena).or_else(|| {
            if prefix_id == self.empty_prefix_id {
                Some(self.no_namespace_id)
            } else {
                None
            }
        })
    }

    fn prefix(&mut self, prefix: &'a str, namespace_uri: &'a str) {
        let prefix_id = self.prefix_lookup.get_id(Prefix::new(prefix));
        let namespace_id = self.namespace_lookup.get_id(Namespace::new(namespace_uri));
        if let Some(element_builder) = &mut self.element_builder {
            element_builder.namespace_info.add(prefix_id, namespace_id);
        }
        // XXX what if element builder is none?
    }

    fn add(&mut self, xml_node: XmlNode<'a>, arena: &mut Arena<XmlNode<'a>>) -> NodeId {
        let node_id = arena.new_node(xml_node);
        self.current_node_id.append(node_id, arena);
        node_id
    }

    fn open_element(&mut self, arena: &mut Arena<XmlNode<'a>>) -> Result<(), Error> {
        let element_builder = self.element_builder.take().unwrap();
        let element = XmlNode::Element(element_builder.into_element(self, arena)?);
        let node_id = self.add(element, arena);
        self.current_node_id = node_id;
        Ok(())
    }

    fn text(&mut self, content: &'a str, arena: &mut Arena<XmlNode<'a>>) {
        self.add(XmlNode::Text(content.into()), arena);
    }

    fn close_element(&mut self, arena: &mut Arena<XmlNode>) {
        let parent_node_id = arena
            .get(self.current_node_id)
            .unwrap()
            .parent()
            .expect("Cannot close root node");
        self.current_node_id = parent_node_id;
    }
}

impl<'a> Document<'a> {
    pub fn parse(xml: &'a str, arena: &mut Arena<XmlNode<'a>>) -> Result<Self, Error> {
        use Token::*;

        let mut builder = DocumentBuilder::new(arena);

        for token in Tokenizer::from(xml) {
            match token? {
                Attribute {
                    prefix,
                    local,
                    value,
                    span,
                } => {
                    if prefix.as_str() == "xmlns" {
                        builder.prefix(local.as_str(), value.as_str());
                    } else {
                    }
                }
                Text { text } => {
                    builder.text(text.as_str(), arena);
                }
                ElementStart {
                    prefix,
                    local,
                    span,
                } => {
                    builder.element(prefix.as_str(), local.as_str());
                }
                ElementEnd { end, span } => {
                    use self::ElementEnd::*;

                    match end {
                        Open => {
                            builder.open_element(arena)?;
                        }
                        Close(prefix, local) => {
                            // XXX check that we're closing the right element
                            builder.close_element(arena);
                        }
                        Empty => {
                            builder.close_element(arena);
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(builder.into_document())
    }
}
