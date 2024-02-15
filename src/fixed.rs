//! A fixed representation of a tree of nodes.
//!
//! Xot trees are mutable, but it is useful to have a fixed representation of a
//! node that you can create and store separately. This has no dependency on
//! the [`Xot`] object. You can turn this fixed representation into a node by
//! calling `.xotify` on them and passing a mutable [`Xot`].
//!
//! Example:
//!
//! ```rust
//! use xot::fixed;
//!
//! let fixed_element = fixed::Element {
//!   name: fixed::Name {
//!     namespace: "".to_string(),
//!     localname: "foo".to_string(),
//!   },
//!   attributes: vec![],
//!   prefixes: vec![],
//!   children: vec![fixed::Content::Text("Example".to_string())],
//! };
//!
//! let mut xot = xot::Xot::new();
//! let node = fixed_element.xotify(&mut xot);
//! assert_eq!(xot.to_string(node).unwrap(), "<foo>Example</foo>");
//! ```

use crate::xotdata::{Node, Xot};

/// A fixed representation of an XML document.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Document {
    /// Comments and processing instructions before the document element
    pub before: Vec<DocumentContent>,
    /// The document element
    pub document_element: Element,
    /// Comments and processing instructions after the document element
    pub after: Vec<DocumentContent>,
}

/// A fixed representation of an XML name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name {
    /// Namespace URI. Empty string means no namespace
    pub namespace: String,
    /// Localname.
    pub localname: String,
}

/// A fixed representation of an XML namespace prefix declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Prefix {
    /// Name of prefix. Empty string means default namespace
    pub name: String,
    /// Namespace URI.
    pub namespace: String,
}

/// A fixed representation of an XML element.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Element {
    /// Name of element
    pub name: Name,
    /// Namespace prefix declarations
    pub prefixes: Vec<Prefix>,
    /// Attributes
    pub attributes: Vec<(Name, String)>,
    /// Children    
    pub children: Vec<Content>,
}

/// A fixed representation of element content
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Content {
    /// A text node
    Text(String),
    /// A comment node
    Comment(String),
    /// A processing instruction node
    ProcessingInstruction(ProcessingInstruction),
    /// An element node
    Element(Element),
}

/// Content that is allowed next to the document element (the root element)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DocumentContent {
    /// A comment node
    Comment(String),
    /// A processing instruction node
    ProcessingInstruction(ProcessingInstruction),
}

/// A fixed representation of a processing instruction
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProcessingInstruction {
    /// Target of processing instruction
    pub target: String,
    /// Content of processing instruction
    pub content: Option<String>,
}

impl Name {
    /// Turn a fixed name into a Xot name
    pub fn xotify(&self, xot: &mut Xot) -> crate::NameId {
        let namespace = xot.add_namespace(&self.namespace);
        xot.add_name_ns(&self.localname, namespace)
    }
}

impl Document {
    /// Turn a fixed document into a Xot node
    pub fn xotify(&self, xot: &mut Xot) -> Node {
        let child = self.document_element.xotify(xot);
        let document = xot.new_document_with_element(child).unwrap();
        for content in &self.before {
            let node = create_document_content_node(xot, content);
            xot.insert_before(child, node).unwrap();
        }
        for content in &self.after {
            let node = create_document_content_node(xot, content);
            xot.append(child, node).unwrap();
        }
        document
    }
}

impl Element {
    /// Turn a fixed element into a Xot node
    pub fn xotify(&self, xot: &mut Xot) -> Node {
        let name = self.name.xotify(xot);
        let prefixes = self
            .prefixes
            .iter()
            .map(|prefix| {
                let prefix_id = xot.add_prefix(&prefix.name);
                let ns_id = xot.add_namespace(&prefix.namespace);
                (prefix_id, ns_id)
            })
            .collect::<Vec<_>>();
        let attributes = self
            .attributes
            .iter()
            .map(|(name, value)| {
                let ns_id = xot.add_namespace(&name.namespace);
                let name_id = xot.add_name_ns(&name.localname, ns_id);
                (name_id, value)
            })
            .collect::<Vec<_>>();

        let element_node = xot.new_element(name);

        let mut namespaces_map = xot.namespaces_mut(element_node);

        for (prefix, ns) in prefixes {
            namespaces_map.insert(prefix, ns);
        }

        let mut attributes_map = xot.attributes_mut(element_node);

        for (name, value) in attributes {
            attributes_map.insert(name, value.clone());
        }

        let children = self
            .children
            .iter()
            .map(|child| child.xotify(xot))
            .collect::<Vec<_>>();
        for child in children {
            xot.append(element_node, child).unwrap();
        }
        element_node
    }
}

impl ProcessingInstruction {
    /// Turn a fixed processing instruction into a Xot node
    pub fn xotify(&self, xot: &mut Xot) -> Node {
        let target = xot.add_name(&self.target);
        xot.new_processing_instruction(target, self.content.as_deref())
    }
}

impl Content {
    /// Turn a fixed content into a Xot node
    fn xotify(&self, xot: &mut Xot) -> Node {
        match self {
            Content::Text(text) => xot.new_text(text),
            Content::Comment(comment) => xot.new_comment(comment),
            Content::ProcessingInstruction(processing_instruction) => {
                processing_instruction.xotify(xot)
            }
            Content::Element(element) => element.xotify(xot),
        }
    }
}

fn create_document_content_node(xot: &mut Xot, content: &DocumentContent) -> Node {
    match content {
        DocumentContent::Comment(comment) => xot.new_comment(comment),
        DocumentContent::ProcessingInstruction(processing_instruction) => {
            processing_instruction.xotify(xot)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xotify() {
        let mut xot = Xot::new();
        let document = Document {
            before: vec![],
            document_element: Element {
                name: Name {
                    namespace: "".to_string(),
                    localname: "foo".to_string(),
                },
                attributes: vec![],
                prefixes: vec![],
                children: vec![Content::Text("Example".to_string())],
            },
            after: vec![],
        };
        let document = document.xotify(&mut xot);
        assert_eq!(xot.to_string(document).unwrap(), "<foo>Example</foo>");
    }
}
