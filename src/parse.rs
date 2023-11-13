use ahash::{HashMap, HashMapExt};
use indextree::NodeId;
use xmlparser::{ElementEnd, StrSpan, Token, Tokenizer};

use crate::encoding::decode;
use crate::entity::{parse_attribute, parse_text};
use crate::error::Error;
use crate::name::{Name, NameId};
use crate::prefix::PrefixId;
use crate::xmlvalue::{Attributes, Comment, Element, Prefixes, ProcessingInstruction, Text, Value};
use crate::xotdata::{Node, Xot};

struct AttributeBuilder {
    prefix: String,
    name: String,
    value: String,
    name_span: Span,
    value_span: Span,
}

struct ElementBuilder {
    prefix: String,
    name: String,
    prefixes: Prefixes,
    attributes: Vec<AttributeBuilder>,
    span: Span,
}

impl ElementBuilder {
    fn new(prefix: StrSpan<'_>, name: StrSpan<'_>) -> Self {
        ElementBuilder {
            prefix: prefix.to_string(),
            name: name.to_string(),
            prefixes: Prefixes::new(),
            attributes: Vec::new(),
            span: Span::from_prefix_name(prefix, name),
        }
    }

    fn build_attributes(
        &mut self,
        document_builder: &mut DocumentBuilder,
        xot: &mut Xot,
    ) -> Result<(Attributes, AttributeSpans), Error> {
        let mut attributes = Attributes::new();
        let mut attribute_spans = Vec::new();
        for attribute_builder in self.attributes.drain(..) {
            let name_id = document_builder.name_id_builder.attribute_name_id(
                &attribute_builder.prefix,
                &attribute_builder.name,
                xot,
            )?;
            attributes.insert(name_id, attribute_builder.value);
            attribute_spans.push((
                name_id,
                attribute_builder.name_span,
                attribute_builder.value_span,
            ));
        }
        Ok((attributes, attribute_spans))
    }

    fn into_element(
        mut self,
        document_builder: &mut DocumentBuilder,
        xot: &mut Xot,
    ) -> Result<(Element, AttributeSpans), Error> {
        document_builder.name_id_builder.push(&self.prefixes);
        let (attributes, attribute_spans) = self.build_attributes(document_builder, xot)?;
        let name_id =
            document_builder
                .name_id_builder
                .element_name_id(&self.prefix, &self.name, xot)?;
        Ok((
            Element {
                name_id,
                prefixes: self.prefixes,
                attributes,
            },
            attribute_spans,
        ))
    }
}

struct DocumentBuilder {
    tree: NodeId,
    current_node_id: NodeId,
    name_id_builder: NameIdBuilder,
    element_builder: Option<ElementBuilder>,
}

impl DocumentBuilder {
    fn new(xot: &mut Xot) -> Self {
        let root = xot.arena.new_node(Value::Root);
        let mut name_id_builder = NameIdBuilder::new(xot.base_prefixes());
        let mut base_prefixes = Prefixes::new();
        base_prefixes.insert(xot.empty_prefix_id, xot.no_namespace_id);
        name_id_builder.push(&base_prefixes);
        DocumentBuilder {
            tree: root,
            current_node_id: root,
            name_id_builder,
            element_builder: None,
        }
    }

    fn element(&mut self, prefix: StrSpan<'_>, name: StrSpan<'_>) {
        self.element_builder = Some(ElementBuilder::new(prefix, name));
    }

    fn prefix(&mut self, prefix: &str, namespace_uri: &str, xot: &mut Xot) {
        let prefix_id = xot.prefix_lookup.get_id_mut(prefix);
        let namespace_id = xot.namespace_lookup.get_id_mut(namespace_uri);
        self.element_builder
            .as_mut()
            .unwrap()
            .prefixes
            .insert(prefix_id, namespace_id);
    }

    fn attribute(
        &mut self,
        prefix: StrSpan<'_>,
        name: StrSpan<'_>,
        value: StrSpan<'_>,
    ) -> Result<(), Error> {
        let attributes = &mut self.element_builder.as_mut().unwrap().attributes;
        let is_duplicate = attributes.iter().any(|attribute_builder| {
            attribute_builder.prefix == prefix.as_str() && attribute_builder.name == name.as_str()
        });
        if is_duplicate {
            let attr_name = if prefix.is_empty() {
                name.to_string()
            } else {
                format!("{}:{}", prefix, name)
            };
            return Err(Error::DuplicateAttribute(attr_name));
        }
        attributes.push(AttributeBuilder {
            prefix: prefix.to_string(),
            name: name.to_string(),
            value: parse_attribute(value.as_str().into())?.to_string(),
            name_span: Span::from_prefix_name(prefix, name),
            value_span: value.into(),
        });
        Ok(())
    }

    fn add(&mut self, value: Value, xot: &mut Xot) -> NodeId {
        let node_id = xot.arena.new_node(value);
        self.current_node_id.append(node_id, &mut xot.arena);
        node_id
    }

    fn open_element(&mut self, xot: &mut Xot) -> Result<(NodeId, Span, AttributeSpans), Error> {
        let element_builder = self.element_builder.take().unwrap();
        let span = element_builder.span;
        let (element, attribute_spans) = element_builder.into_element(self, xot)?;
        let element = Value::Element(element);
        let node_id = self.add(element, xot);
        self.current_node_id = node_id;
        Ok((node_id, span, attribute_spans))
    }

    fn text(&mut self, content: &str, xot: &mut Xot) -> Result<NodeId, Error> {
        let content = parse_text(content.into())?;
        Ok(self.add(Value::Text(Text::new(content.to_string())), xot))
    }

    fn cdata_text(&mut self, content: &str, xot: &mut Xot) -> Result<(), Error> {
        self.add(Value::Text(Text::new(content.to_string())), xot);
        Ok(())
    }

    fn close_element_immediate(&mut self, xot: &mut Xot) -> NodeId {
        let current_node = xot.arena.get(self.current_node_id).unwrap();
        if let Value::Element(element) = current_node.get() {
            self.name_id_builder.pop(&element.prefixes);
        }
        let closed_node_id = self.current_node_id;
        self.current_node_id = current_node.parent().expect("Cannot close root node");
        closed_node_id
    }

    fn close_element(&mut self, prefix: &str, name: &str, xot: &mut Xot) -> Result<NodeId, Error> {
        let name_id = self.name_id_builder.element_name_id(prefix, name, xot)?;
        let current_node = xot.arena.get(self.current_node_id).unwrap();
        if let Value::Element(element) = current_node.get() {
            if element.name_id != name_id {
                return Err(Error::InvalidCloseTag(prefix.to_string(), name.to_string()));
            }
            self.name_id_builder.pop(&element.prefixes);
        }
        let closed_node_id = self.current_node_id;
        self.current_node_id = current_node.parent().expect("Cannot close root node");
        Ok(closed_node_id)
    }

    fn comment(&mut self, content: &str, xot: &mut Xot) -> Result<NodeId, Error> {
        // XXX are there illegal comments, like those with -- inside? or
        // won't they pass the parser?
        Ok(self.add(Value::Comment(Comment::new(content.to_string())), xot))
    }

    fn processing_instruction(
        &mut self,
        target: &str,
        content: Option<&str>,
        xot: &mut Xot,
    ) -> Result<NodeId, Error> {
        // XXX are there illegal processing instructions, like those with
        // ?> inside? or won't they pass the parser?
        Ok(self.add(
            Value::ProcessingInstruction(ProcessingInstruction::new(
                target.to_string(),
                content.map(|s| s.to_string()),
            )),
            xot,
        ))
    }

    fn is_current_node_root(&self, xot: &Xot) -> bool {
        matches!(xot.arena[self.current_node_id].get(), Value::Root)
    }
}

struct NameIdBuilder {
    namespace_stack: Vec<Prefixes>,
}

impl NameIdBuilder {
    fn new(prefixes: Prefixes) -> Self {
        let namespace_stack = vec![prefixes];
        Self { namespace_stack }
    }

    fn push(&mut self, prefixes: &Prefixes) {
        if prefixes.is_empty() {
            return;
        }
        // can always use top as there's a bottom entry
        let mut entry = self.top().clone();
        entry.extend(prefixes);
        self.namespace_stack.push(entry);
    }

    fn pop(&mut self, prefixes: &Prefixes) {
        if prefixes.is_empty() {
            return;
        }
        // should always be able to pop as there's a bottom entry
        self.namespace_stack.pop();
    }

    #[inline]
    fn top(&self) -> &Prefixes {
        &self.namespace_stack[self.namespace_stack.len() - 1]
    }

    fn element_name_id(
        &mut self,
        prefix: &str,
        name: &str,
        xot: &mut Xot,
    ) -> Result<NameId, Error> {
        let prefix_id = xot.prefix_lookup.get_id_mut(prefix);
        if let Ok(name_id) = self.name_id_with_prefix_id(prefix_id, name, xot) {
            Ok(name_id)
        } else {
            Err(Error::UnknownPrefix(prefix.to_string()))
        }
    }

    fn attribute_name_id(
        &mut self,
        prefix: &str,
        name: &str,
        xot: &mut Xot,
    ) -> Result<NameId, Error> {
        // an unprefixed attribute is in no namespace, not
        // in the default namespace
        // https://stackoverflow.com/questions/3312390/xml-default-namespaces-for-unqualified-attribute-names
        let prefix_id = xot.prefix_lookup.get_id_mut(prefix);
        if prefix_id == xot.empty_prefix_id {
            let name = Name::new(name.to_string(), xot.no_namespace_id);
            return Ok(xot.name_lookup.get_id_mut(&name));
        }
        if let Ok(name_id) = self.name_id_with_prefix_id(prefix_id, name, xot) {
            Ok(name_id)
        } else {
            Err(Error::UnknownPrefix(prefix.to_string()))
        }
    }

    fn name_id_with_prefix_id(
        &mut self,
        prefix_id: PrefixId,
        name: &str,
        xot: &mut Xot,
    ) -> Result<NameId, ()> {
        let namespace_id = if !self.namespace_stack.is_empty() {
            self.top().get(&prefix_id)
        } else {
            None
        };
        let namespace_id = namespace_id.ok_or(())?;
        let name = Name::new(name.to_string(), *namespace_id);
        Ok(xot.name_lookup.get_id_mut(&name))
    }
}

/// A span with a start and end position
///
/// Spans describe ranges in the source text, with the end point not inclusive,
/// like a range. It's not a `std::ops::Range` as it's handy for a span to be
/// `Copy`.
///
/// You can obtain these from a [`SpanInfo`](crate::SpanInfo). You create a
/// [`SpanInfo`] by using
/// [`Xot::parse_with_span_info`](crate::Xot::parse_with_span_info).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Span {
    /// the start position in the XML source
    pub start: usize,
    /// the end position in the XML source
    pub end: usize,
}

impl Span {
    /// Construct a new span
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }

    fn from_prefix_name(prefix: StrSpan<'_>, name: StrSpan<'_>) -> Self {
        if prefix.is_empty() {
            Self::new(name.start(), name.end())
        } else {
            Self::new(prefix.start(), name.end())
        }
    }

    /// Turn a span into a range
    pub fn range(&self) -> std::ops::Range<usize> {
        self.start..self.end
    }
}

impl<'a> From<xmlparser::StrSpan<'a>> for Span {
    fn from(span: xmlparser::StrSpan) -> Self {
        Span {
            start: span.start(),
            end: span.end(),
        }
    }
}

type AttributeSpans = Vec<(NameId, Span, Span)>;

/// A key to use to look up span information using
/// [`SpanInfo::get`](`crate::SpanInfo::get`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpanInfoKey {
    /// The name part of an attribute.
    /// In `foo:name="value"`, the `foo:name` part
    AttributeName(Node, NameId),
    /// The value part of an attribute.
    /// In `foo:name="value"`, the `value` part
    AttributeValue(Node, NameId),
    /// The name part of a start element tag.
    /// In `<foo:name ..>`, the `foo:name` part
    ElementStart(Node),
    /// The closing part of the end element tag (or a self-closing element).
    /// In `</foo:name>`, the `</foo:name>` part, or if it is an empty element
    /// `<foo:name/>`, the `/>` part
    ElementEnd(Node),
    /// Text node.
    /// In `<foo>text</foo>`, the `text` part
    Text(Node),
    /// Comment node.
    /// In `<!--comment-->`, the `comment` part
    Comment(Node),
    /// The target part of a processing instruction.
    /// In `<?target content?>`, the `target` part
    PiTarget(Node),
    /// The content part of a processing instruction (if defined).
    /// In `<?target content?>`, the `content` part
    PiContent(Node),
}

/// Span information for a parsed XML document.
///
/// This span information is valid immediately after the parse. It becomes
/// invalid as soon as you mutate the parsed document.
///
/// You obtain this by using
/// [`Xot::parse_with_span_info`](`Xot::parse_with_span_info`).
///
/// You use a [`SpanInfoKey`](crate::SpanInfoKey) to look up the span
/// information.
pub struct SpanInfo {
    map: HashMap<SpanInfoKey, Span>,
}

impl SpanInfo {
    fn new() -> Self {
        SpanInfo {
            map: HashMap::new(),
        }
    }

    /// Get span info by [`SpanInfoKey`](crate::SpanInfoKey)
    pub fn get(&self, key: SpanInfoKey) -> Option<&Span> {
        self.map.get(&key)
    }

    fn add(&mut self, key: SpanInfoKey, span: Span) {
        self.map.insert(key, span);
    }

    fn add_attribute_spans(&mut self, node_id: NodeId, attribute_spans: AttributeSpans) {
        for (attribute_name, name_span, value_span) in attribute_spans {
            self.add(
                SpanInfoKey::AttributeName(node_id.into(), attribute_name),
                name_span,
            );
            self.add(
                SpanInfoKey::AttributeValue(node_id.into(), attribute_name),
                value_span,
            );
        }
    }
}

/// ## Parsing
impl Xot {
    /// Parse a string containing XML into a node. Retain span information.
    ///
    /// This parses the XML source into a Xot tree, and also returns
    /// [`SpanInfo`](`crate::SpanInfo`) which describes where nodes in the
    /// tree are located in the source text.
    pub fn parse_with_span_info(&mut self, xml: &str) -> Result<(Node, SpanInfo), Error> {
        use Token::*;

        let mut builder = DocumentBuilder::new(self);
        let mut span_info = SpanInfo::new();
        for token in Tokenizer::from(xml) {
            match token? {
                Attribute {
                    prefix,
                    local,
                    value,
                    span: _,
                } => {
                    if prefix.as_str() == "xmlns" {
                        builder.prefix(local.as_str(), value.as_str(), self);
                    } else if local.as_str() == "xmlns" {
                        builder.prefix("", value.as_str(), self);
                    } else {
                        builder.attribute(prefix, local, value)?;
                    }
                }
                Text { text } => {
                    let node_id = builder.text(text.as_str(), self)?;
                    span_info.add(SpanInfoKey::Text(node_id.into()), text.into());
                }
                ElementStart {
                    prefix,
                    local,
                    span: _,
                } => {
                    builder.element(prefix, local);
                }

                ElementEnd {
                    end,
                    span: end_span,
                } => {
                    use self::ElementEnd::*;

                    match end {
                        Open => {
                            let (node_id, span, attribute_spans) = builder.open_element(self)?;
                            span_info.add(SpanInfoKey::ElementStart(node_id.into()), span);
                            span_info.add_attribute_spans(node_id, attribute_spans);
                        }
                        Close(prefix, local) => {
                            let node_id =
                                builder.close_element(prefix.as_str(), local.as_str(), self)?;
                            span_info.add(SpanInfoKey::ElementEnd(node_id.into()), end_span.into());
                        }
                        Empty => {
                            let (node_id, span, attribute_spans) = builder.open_element(self)?;
                            span_info.add(SpanInfoKey::ElementStart(node_id.into()), span);
                            span_info.add_attribute_spans(node_id, attribute_spans);
                            let node_id = builder.close_element_immediate(self);
                            span_info.add(SpanInfoKey::ElementEnd(node_id.into()), end_span.into());
                        }
                    }
                }
                Comment { text, span: _ } => {
                    let node_id = builder.comment(text.as_str(), self)?;
                    span_info.add(SpanInfoKey::Comment(node_id.into()), text.into());
                }
                ProcessingInstruction {
                    target,
                    content,
                    span: _,
                } => {
                    let node_id = builder.processing_instruction(
                        target.as_str(),
                        content.map(|s| s.as_str()),
                        self,
                    )?;
                    span_info.add(SpanInfoKey::PiTarget(node_id.into()), target.into());
                    if let Some(content) = content {
                        span_info.add(SpanInfoKey::PiContent(node_id.into()), content.into());
                    }
                }
                Declaration {
                    version,
                    encoding: _,
                    standalone,
                    span: _,
                } => {
                    if version.as_str() != "1.0" {
                        return Err(Error::UnsupportedVersion(version.to_string()));
                    }
                    if let Some(standalone) = standalone {
                        if !standalone {
                            return Err(Error::UnsupportedNotStandalone);
                        }
                    }
                }
                Cdata { text, span: _ } => {
                    builder.cdata_text(text.as_str(), self)?;
                }
                DtdStart { .. } => {
                    return Err(Error::DtdUnsupported);
                }
                DtdEnd { .. } => {
                    return Err(Error::DtdUnsupported);
                }
                EmptyDtd { .. } => {
                    return Err(Error::DtdUnsupported);
                }
                EntityDeclaration { .. } => {
                    return Err(Error::DtdUnsupported);
                }
            }
        }

        if builder.is_current_node_root(self) {
            Ok((Node::new(builder.tree), span_info))
        } else {
            Err(Error::UnclosedTag)
        }
    }

    /// Parse a string containing XML into a node.
    ///
    /// Even though the encoding in the XML declaration may indicate otherwise,
    /// the string is interpreted as a Rust string, i.e. UTF-8. If you need to
    /// decode the string first, use [`Xot::parse_bytes`].
    ///
    /// The returned node is the root node of the
    /// parsed XML document.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse("<hello/>")?;
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn parse(&mut self, xml: &str) -> Result<Node, Error> {
        self.parse_with_span_info(xml).map(|(node, _)| node)
    }

    /// Parse bytes containing XML into a node.
    ///
    /// This attempts to decode the data in the bytes into a Rust string
    /// (UTF-8) first, then parses this string.
    ///
    /// If you already have a Rust string, use [`Xot::parse`].
    ///
    /// The returned node is the root node of the parsed XML document.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse_bytes(b"<hello/>")?;
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    ///
    /// let root = xot.parse_bytes(b"<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?><p>\xe9</p>")?;
    ///
    /// let doc_el = xot.document_element(root)?;
    /// let txt_value = xot.text_content_str(doc_el).unwrap();
    /// assert_eq!(txt_value, "é");
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn parse_bytes(&mut self, bytes: &[u8]) -> Result<Node, Error> {
        let xml = decode(bytes, None);
        self.parse(&xml)
    }
}
