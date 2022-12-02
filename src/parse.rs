use id_tree::{InsertBehavior, Node, NodeId};
use xmlparser::{ElementEnd, Token, Tokenizer};

use crate::document::{namespace_by_prefix, Document, XmlTree};
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
    ) -> Result<Element<'a>, Error> {
        let prefix_id = document_builder
            .prefix_lookup
            .get_id(Prefix::new(self.prefix));
        let namespace_id = self
            .namespace_info
            .to_namespace
            .get(&prefix_id)
            .copied()
            .or_else(|| document_builder.namespace_by_prefix(prefix_id));
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
    tree: XmlTree<'a>,
    current_node_id: Option<NodeId>,
    element_builder: Option<ElementBuilder<'a>>,
}

impl<'a> DocumentBuilder<'a> {
    fn new() -> Self {
        let mut namespace_lookup = NamespaceLookup::new();
        // XXX absence of namespace is defined as the empty namespace,
        // we should forbid its construction otherwise?
        let no_namespace_id = namespace_lookup.get_id(Namespace::new(""));

        let mut prefix_lookup = PrefixLookup::new();
        let empty_prefix_id = prefix_lookup.get_id(Prefix::new(""));

        DocumentBuilder {
            namespace_lookup,
            prefix_lookup,
            no_namespace_id,
            empty_prefix_id,
            name_lookup: NameLookup::new(),
            tree: XmlTree::new(),
            current_node_id: None,
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

    fn namespace_by_prefix(&self, prefix_id: PrefixId) -> Option<NamespaceId> {
        self.current_node_id
            .as_ref()
            .and_then(|node_id| namespace_by_prefix(&self.tree, node_id, prefix_id).unwrap())
            .or_else(|| {
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

    fn add(&mut self, xml_node: XmlNode<'a>) -> Result<NodeId, Error> {
        let behavior = match self.current_node_id {
            Some(ref current_node_id) => InsertBehavior::UnderNode(current_node_id),
            None => InsertBehavior::AsRoot,
        };
        let node_id = self.tree.insert(Node::new(xml_node), behavior)?;
        Ok(node_id)
    }

    fn open_element(&mut self) -> Result<(), Error> {
        let element_builder = self.element_builder.take().unwrap();
        let element = XmlNode::Element(element_builder.into_element(self)?);
        let node_id = self.add(element)?;
        self.current_node_id = Some(node_id);
        Ok(())
    }

    fn text(&mut self, content: &'a str) -> Result<(), Error> {
        self.add(XmlNode::Text(content.into()))?;
        Ok(())
    }

    fn close_element(&mut self) {
        // XXX what if empty without open?
        let current_node_id = self.current_node_id.take().unwrap();
        // we have to clone the node id here, but don't maintain other references
        self.current_node_id = self.tree.get(&current_node_id).unwrap().parent().cloned();
    }
}

impl<'a> Document<'a> {
    pub fn parse(xml: &'a str) -> Result<Self, Error> {
        use Token::*;

        let mut builder = DocumentBuilder::new();

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
                    builder.text(text.as_str())?;
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
                            builder.open_element()?;
                        }
                        Close(prefix, local) => {
                            // XXX check that we're closing the right element
                            builder.close_element();
                        }
                        Empty => {
                            builder.close_element();
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(builder.into_document())
    }
}
