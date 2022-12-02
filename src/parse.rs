use indextree::NodeId;
use xmlparser::{ElementEnd, Token, Tokenizer};

use crate::document::{namespace_by_prefix, Document, XmlData};
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
            .data
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
        let name_id = document_builder.data.name_lookup.get_id(name);
        Ok(Element {
            name_id,
            namespace_info: self.namespace_info,
            attributes: self.attributes,
        })
    }
}

struct DocumentBuilder<'a> {
    data: &'a mut XmlData<'a>,
    tree: NodeId,
    current_node_id: NodeId,
    element_builder: Option<ElementBuilder<'a>>,
}

impl<'a> DocumentBuilder<'a> {
    fn new(data: &'a mut XmlData<'a>) -> Self {
        let root = data.arena.new_node(XmlNode::Root);
        DocumentBuilder {
            data,
            tree: root,
            current_node_id: root,
            element_builder: None,
        }
    }

    fn into_document(self) -> Document<'a> {
        Document {
            data: self.data,
            tree: self.tree,
        }
    }

    fn element(&mut self, prefix: &'a str, name: &'a str) {
        self.element_builder = Some(ElementBuilder::new(prefix, name));
    }

    fn namespace_by_prefix(&self, prefix_id: PrefixId) -> Option<NamespaceId> {
        namespace_by_prefix(self.current_node_id, prefix_id, &self.data.arena).or_else(|| {
            if prefix_id == self.data.empty_prefix_id {
                Some(self.data.no_namespace_id)
            } else {
                None
            }
        })
    }

    fn prefix(&mut self, prefix: &'a str, namespace_uri: &'a str) {
        let prefix_id = self.data.prefix_lookup.get_id(Prefix::new(prefix));
        let namespace_id = self
            .data
            .namespace_lookup
            .get_id(Namespace::new(namespace_uri));
        if let Some(element_builder) = &mut self.element_builder {
            element_builder.namespace_info.add(prefix_id, namespace_id);
        }
        // XXX what if element builder is none?
    }

    fn add(&mut self, xml_node: XmlNode<'a>) -> NodeId {
        let node_id = self.data.arena.new_node(xml_node);
        self.current_node_id.append(node_id, &mut self.data.arena);
        node_id
    }

    fn open_element(&mut self) -> Result<(), Error> {
        let element_builder = self.element_builder.take().unwrap();
        let element = XmlNode::Element(element_builder.into_element(self)?);
        let node_id = self.add(element);
        self.current_node_id = node_id;
        Ok(())
    }

    fn text(&mut self, content: &'a str) {
        self.add(XmlNode::Text(content.into()));
    }

    fn close_element(&mut self) {
        let parent_node_id = self
            .data
            .arena
            .get(self.current_node_id)
            .unwrap()
            .parent()
            .expect("Cannot close root node");
        self.current_node_id = parent_node_id;
    }
}

impl<'a> Document<'a> {
    pub fn parse(xml: &'a str, data: &'a mut XmlData<'a>) -> Result<Self, Error> {
        use Token::*;

        let mut builder = DocumentBuilder::new(data);

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
                    builder.text(text.as_str());
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
