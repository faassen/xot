use ahash::{HashMap, HashMapExt};
use indextree::NodeId;
use xmlparser::{ElementEnd, StrSpan, Token, Tokenizer};

use crate::encoding::decode;
use crate::entity::{parse_attribute, parse_text};
use crate::error::ParseError;
use crate::id::{Name, NameId, PrefixId};
use crate::xmlvalue::{Attribute, Comment, Element, Namespace, ProcessingInstruction, Text, Value};
use crate::xotdata::{Node, Xot};
use crate::NamespaceId;

type Namespaces = Vec<(PrefixId, NamespaceId)>;

struct AttributeBuilder {
    prefix: String,
    name: String,
    value: String,
    name_span: Span,
    value_span: Span,
    prefix_span: Span,
}

struct ElementBuilder {
    prefix: String,
    name: String,
    namespaces: Namespaces,
    attributes: Vec<AttributeBuilder>,
    prefix_span: Span,
    span: Span,
}

impl ElementBuilder {
    fn new(prefix: StrSpan<'_>, name: StrSpan<'_>) -> Self {
        ElementBuilder {
            prefix: prefix.to_string(),
            name: name.to_string(),
            namespaces: Namespaces::new(),
            attributes: Vec::new(),
            prefix_span: prefix.into(),
            span: Span::from_prefix_name(prefix, name),
        }
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
        let document = xot.arena.new_node(Value::Document);
        let mut name_id_builder = NameIdBuilder::new(xot.base_prefixes().into_iter().collect());
        let base_prefixes = vec![(xot.empty_prefix_id, xot.no_namespace_id)];
        name_id_builder.push(base_prefixes);
        DocumentBuilder {
            tree: document,
            current_node_id: document,
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
            .namespaces
            .push((prefix_id, namespace_id));
    }

    fn attribute(
        &mut self,
        prefix: StrSpan<'_>,
        name: StrSpan<'_>,
        value: StrSpan<'_>,
    ) -> Result<(), ParseError> {
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
            let span = Span::from_prefix_name(prefix, name);
            return Err(ParseError::DuplicateAttribute(attr_name, span));
        }
        let value_span = value.into();
        let value = parse_attribute(value.as_str().into(), value.start())?.to_string();
        // if this is an xml:id we want to apply xml:id normalization as described here
        // https://www.w3.org/TR/xml-id/#id-avn
        let value = if name == "id" && prefix == "xml" {
            normalize_xml_id(&value)
        } else {
            value
        };
        attributes.push(AttributeBuilder {
            prefix: prefix.to_string(),
            name: name.to_string(),
            value,
            name_span: Span::from_prefix_name(prefix, name),
            value_span,
            prefix_span: prefix.into(),
        });
        Ok(())
    }

    fn add(&mut self, value: Value, xot: &mut Xot) -> NodeId {
        let node_id = xot.arena.new_node(value);
        self.current_node_id.append(node_id, &mut xot.arena);
        node_id
    }

    fn open_element(
        &mut self,
        xot: &mut Xot,
    ) -> Result<(NodeId, Span, AttributeSpans), ParseError> {
        let element_builder = self.element_builder.take().unwrap();
        let span = element_builder.span;

        self.name_id_builder
            .push(element_builder.namespaces.clone());

        let name_id = self.name_id_builder.element_name_id(
            &element_builder.prefix,
            &element_builder.name,
            element_builder.prefix_span,
            xot,
        )?;
        let element_value = Value::Element(Element { name_id });
        let node_id = self.add(element_value, xot);
        self.current_node_id = node_id;

        // add namespace nodes
        for (prefix_id, namespace_id) in &element_builder.namespaces {
            let namespace_node = xot.arena.new_node(Value::Namespace(Namespace {
                prefix_id: *prefix_id,
                namespace_id: *namespace_id,
            }));
            self.current_node_id.append(namespace_node, &mut xot.arena);
        }
        // add attribute nodes
        let mut attribute_spans = Vec::new();
        for attribute_builder in element_builder.attributes {
            let name_id = self.name_id_builder.attribute_name_id(
                &attribute_builder.prefix,
                &attribute_builder.name,
                attribute_builder.prefix_span,
                xot,
            )?;
            let attribute_node = xot.arena.new_node(Value::Attribute(Attribute {
                name_id,
                value: attribute_builder.value,
            }));
            attribute_spans.push((
                name_id,
                attribute_builder.name_span,
                attribute_builder.value_span,
            ));
            self.current_node_id.append(attribute_node, &mut xot.arena);
        }

        Ok((node_id, span, attribute_spans))
    }

    // consolidates a text node with previous node if possible. If consolidation
    // took place returns the node id , otherwise none.
    fn consolidate_text(&mut self, content: &str, xot: &mut Xot) -> Option<NodeId> {
        // let's look at the last node we added
        let last = xot.arena[self.current_node_id].last_child();
        if let Some(last) = last {
            let value = xot.arena.get_mut(last).unwrap().get_mut();
            if let Value::Text(last_text) = value {
                last_text.get_mut().push_str(content);
                return Some(last);
            }
        }
        None
    }

    fn text(&mut self, content: &StrSpan, xot: &mut Xot) -> Result<NodeId, ParseError> {
        let content = parse_text(content.as_str().into(), content.start())?;
        if let Some(last) = self.consolidate_text(&content, xot) {
            return Ok(last);
        }
        Ok(self.add(Value::Text(Text::new(content.to_string())), xot))
    }

    fn cdata_text(&mut self, content: &str, xot: &mut Xot) -> Result<NodeId, ParseError> {
        if let Some(last) = self.consolidate_text(content, xot) {
            return Ok(last);
        }
        Ok(self.add(Value::Text(Text::new(content.to_string())), xot))
    }

    fn close_element_immediate(&mut self, xot: &mut Xot) -> NodeId {
        let current_node = xot.arena.get(self.current_node_id).unwrap();
        if matches!(current_node.get(), Value::Element(_)) {
            self.name_id_builder.pop();
        }
        let closed_node_id = self.current_node_id;
        self.current_node_id = current_node.parent().expect("Cannot close document node");
        closed_node_id
    }

    fn close_element(
        &mut self,
        prefix: StrSpan,
        name: StrSpan,
        xot: &mut Xot,
    ) -> Result<NodeId, ParseError> {
        let name_id = self
            .name_id_builder
            .element_name_id(&prefix, &name, prefix.into(), xot)?;
        let current_node = xot.arena.get(self.current_node_id).unwrap();
        if let Value::Element(element) = current_node.get() {
            if element.name_id != name_id {
                return Err(ParseError::InvalidCloseTag(
                    prefix.to_string(),
                    name.to_string(),
                    Span::from_prefix_name(prefix, name),
                ));
            }
            self.name_id_builder.pop();
        }
        let closed_node_id = self.current_node_id;
        self.current_node_id = current_node.parent().expect("Cannot close document node");
        Ok(closed_node_id)
    }

    fn comment(&mut self, content: &str, xot: &mut Xot) -> Result<NodeId, ParseError> {
        // XXX are there illegal comments, like those with -- inside? or
        // won't they pass the parser?
        Ok(self.add(Value::Comment(Comment::new(content.to_string())), xot))
    }

    fn processing_instruction(
        &mut self,
        target: &str,
        content: Option<&str>,
        xot: &mut Xot,
    ) -> Result<NodeId, ParseError> {
        // XXX are there illegal processing instructions, like those with
        // ?> inside? or won't they pass the parser? What about those with xml?
        let target = xot.add_name(target);
        Ok(self.add(
            Value::ProcessingInstruction(ProcessingInstruction::new(
                target,
                content.map(|s| s.to_string()),
            )),
            xot,
        ))
    }

    fn is_current_node_document(&self, xot: &Xot) -> bool {
        matches!(xot.arena[self.current_node_id].get(), Value::Document)
    }
}

struct NameIdBuilder {
    namespace_stack: Vec<Namespaces>,
}

impl NameIdBuilder {
    fn new(prefixes: Namespaces) -> Self {
        let namespace_stack = vec![prefixes];
        Self { namespace_stack }
    }

    fn push(&mut self, namespaces: Namespaces) {
        self.namespace_stack.push(namespaces);
    }

    fn pop(&mut self) {
        // should always be able to pop as there's a bottom entry
        self.namespace_stack.pop();
    }

    fn element_name_id(
        &mut self,
        prefix: &str,
        name: &str,
        prefix_span: Span,
        xot: &mut Xot,
    ) -> Result<NameId, ParseError> {
        let prefix_id = xot.prefix_lookup.get_id_mut(prefix);
        if let Ok(name_id) = self.name_id_with_prefix_id(prefix_id, name, xot) {
            Ok(name_id)
        } else {
            Err(ParseError::UnknownPrefix(prefix.to_string(), prefix_span))
        }
    }

    fn attribute_name_id(
        &mut self,
        prefix: &str,
        name: &str,
        prefix_span: Span,
        xot: &mut Xot,
    ) -> Result<NameId, ParseError> {
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
            Err(ParseError::UnknownPrefix(prefix.to_string(), prefix_span))
        }
    }

    fn name_id_with_prefix_id(
        &mut self,
        prefix_id: PrefixId,
        name: &str,
        xot: &mut Xot,
    ) -> Result<NameId, ()> {
        // go through namespace stack backwards, find the first namespace
        // that matches this prefix
        let namespace_id = self.namespace_stack.iter().rev().find_map(|ns| {
            ns.iter()
                .rev()
                .find_map(|(p, ns)| if *p == prefix_id { Some(*ns) } else { None })
        });
        let namespace_id = namespace_id.ok_or(())?;
        let name = Name::new(name.to_string(), namespace_id);
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

impl From<xmlparser::StrSpan<'_>> for Span {
    fn from(span: xmlparser::StrSpan) -> Self {
        Span {
            start: span.start(),
            end: span.end(),
        }
    }
}

impl From<&xmlparser::StrSpan<'_>> for Span {
    fn from(span: &xmlparser::StrSpan) -> Self {
        Span {
            start: span.start(),
            end: span.end(),
        }
    }
}

impl From<Span> for std::ops::Range<usize> {
    fn from(span: Span) -> Self {
        span.range()
    }
}

impl From<std::ops::Range<usize>> for Span {
    fn from(range: std::ops::Range<usize>) -> Self {
        Span {
            start: range.start,
            end: range.end,
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
///
/// The span of a CDATA section is only its text content:
///
/// ```text
/// <p><![CDATA[content]]></p>
///             ^^^^^^^
/// ```
///
/// There is an exception to this. During parsing, adjacent CDATA sections and
/// text nodes are consolidated into a single text node. This text node has the
/// span starting with the first adjacent CDATA section or text node and ending
/// with the last adjacent CDATA section or text node.
///
/// Example:
///
/// ```text
/// <p>text<![CDATA[content]]>text</p>
///    ^^^^^^^^^^^^^^^^^^^^^^^^^^^
/// ```
///
/// This can lead to the slightly odd situation where only part of the CDATA
/// marker is included in the span:
///
/// ```text
/// <p>text<![CDATA[content]]>foo</p>
///                 ^^^^^^^^^^^^^
///```
///
/// In every case all text content in the adjacent CDATA and text is included
/// in the span.
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

    fn extend_text_span(&mut self, node: Node, span: Span) {
        // if we already have span for this (text) node it, we need to store the span with that
        // start and the given ending
        let key = SpanInfoKey::Text(node);
        if let Some(existing_span) = self.map.get(&key) {
            let start = existing_span.start;
            let end = span.end;
            self.map.insert(key, Span::new(start, end));
        } else {
            self.map.insert(key, span);
        }
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
    /// Parse a string containing XML into a document node. Retain span information.
    ///
    /// This parses the XML source into a Xot tree, and also returns
    /// [`SpanInfo`](`crate::SpanInfo`) which describes where nodes in the
    /// tree are located in the source text.
    pub fn parse_with_span_info(&mut self, xml: &str) -> Result<(Node, SpanInfo), ParseError> {
        let tokenizer = Tokenizer::from(xml);
        let (span_info, builder) = self._parse(tokenizer)?;
        // we expect both a document as the current node (everything else being
        // closed) *and* the content of this node containing a single element
        // if not, we have a problem. We want to produce a parse error for
        // this, as this is the parser.
        if builder.is_current_node_document(self) {
            let document_node = Node::new(builder.tree);
            let mut element_nodes = Vec::new();

            for child in self.children(document_node) {
                match self.value(child) {
                    Value::Element(_) => element_nodes.push(child),
                    Value::Text(_) => {
                        return Err(ParseError::TextAtTopLevel(
                            *span_info.get(SpanInfoKey::Text(child)).unwrap(),
                        ));
                    }
                    _ => {}
                }
            }
            if element_nodes.is_empty() {
                return Err(ParseError::NoElementAtTopLevel(xml.len()));
            }
            if element_nodes.len() > 1 {
                return Err(ParseError::MultipleElementsAtTopLevel(
                    *span_info
                        .get(SpanInfoKey::ElementStart(element_nodes[1]))
                        .unwrap(),
                ));
            }
            Ok((document_node, span_info))
        } else {
            let current_node = Node::new(builder.current_node_id);

            // the top level node's span is the problem
            Err(ParseError::UnclosedTag(
                *span_info
                    .get(SpanInfoKey::ElementStart(current_node))
                    .unwrap(),
            ))
        }
    }

    /// Parse a string containing an XML fragment into a document node.
    ///
    /// This is similar to [`Xot::parse``], but it relaxes the well-formedness
    /// requirements. Specifically, it allows text nodes at the top level, and
    /// does not require a document element, and allows multiple elements. In
    /// short, a fragment behaves like an element (without name, attributes or
    /// namespace definitions).
    ///
    /// This is to support
    /// <https://www.w3.org/TR/xpath-datamodel/#DocumentNode> which is more
    /// permissive than standard XML.
    ///
    /// This parses the XML source into a Xot tree, and also returns
    /// [`SpanInfo`](`crate::SpanInfo`) which describes where nodes in the
    /// tree are located in the source text.
    pub fn parse_fragment_with_span_info(
        &mut self,
        xml: &str,
    ) -> Result<(Node, SpanInfo), ParseError> {
        let tokenizer = Tokenizer::from_fragment(xml, 0..xml.len());
        let (span_info, builder) = self._parse(tokenizer)?;
        if builder.is_current_node_document(self) {
            let document_node = Node::new(builder.tree);
            Ok((document_node, span_info))
        } else {
            let current_node = Node::new(builder.current_node_id);

            // the top level node's span is the problem
            Err(ParseError::UnclosedTag(
                *span_info
                    .get(SpanInfoKey::ElementStart(current_node))
                    .unwrap(),
            ))
        }
    }

    fn _parse(
        &mut self,
        mut tokenizer: Tokenizer<'_>,
    ) -> Result<(SpanInfo, DocumentBuilder), ParseError> {
        use Token::*;

        let mut builder = DocumentBuilder::new(self);
        let mut span_info = SpanInfo::new();

        let mut position;
        loop {
            // getting the position unconditionally is required to get
            // the right one for the error handling, which is a bit unfortunate
            // https://github.com/RazrFalcon/xmlparser/issues/30
            position = tokenizer.stream().pos();
            if let Some(token) = tokenizer.next() {
                let token = match token {
                    Ok(token) => token,
                    Err(e) => {
                        return Err(ParseError::XmlParser(e, position));
                    }
                };
                match token {
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
                        let node_id = builder.text(&text, self)?;
                        span_info.extend_text_span(node_id.into(), text.into());
                    }
                    Cdata { text, span: _ } => {
                        let node_id = builder.cdata_text(text.as_str(), self)?;
                        span_info.extend_text_span(node_id.into(), text.into());
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
                                let (node_id, span, attribute_spans) =
                                    builder.open_element(self)?;
                                span_info.add(SpanInfoKey::ElementStart(node_id.into()), span);
                                span_info.add_attribute_spans(node_id, attribute_spans);
                            }
                            Close(prefix, local) => {
                                let node_id = builder.close_element(prefix, local, self)?;
                                span_info
                                    .add(SpanInfoKey::ElementEnd(node_id.into()), end_span.into());
                            }
                            Empty => {
                                let (node_id, span, attribute_spans) =
                                    builder.open_element(self)?;
                                span_info.add(SpanInfoKey::ElementStart(node_id.into()), span);
                                span_info.add_attribute_spans(node_id, attribute_spans);
                                let node_id = builder.close_element_immediate(self);
                                span_info
                                    .add(SpanInfoKey::ElementEnd(node_id.into()), end_span.into());
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
                        span,
                    } => {
                        if version.as_str() != "1.0" {
                            return Err(ParseError::UnsupportedVersion(
                                version.to_string(),
                                version.into(),
                            ));
                        }
                        if let Some(standalone) = standalone {
                            if !standalone {
                                return Err(ParseError::UnsupportedNotStandalone(span.into()));
                            }
                        }
                    }
                    DtdStart { span, .. } => {
                        return Err(ParseError::DtdUnsupported(span.into()));
                    }
                    DtdEnd { span, .. } => {
                        return Err(ParseError::DtdUnsupported(span.into()));
                    }
                    EmptyDtd { span, .. } => {
                        return Err(ParseError::DtdUnsupported(span.into()));
                    }
                    EntityDeclaration { span, .. } => {
                        return Err(ParseError::DtdUnsupported(span.into()));
                    }
                }
            } else {
                return Ok((span_info, builder));
            }
        }
    }

    /// Parse a string containing XML into a document node.
    ///
    /// Even though the encoding in the XML declaration may indicate otherwise,
    /// the string is interpreted as a Rust string, i.e. UTF-8. If you need to
    /// decode the string first, use [`Xot::parse_bytes`].
    ///
    /// The returned node is the document node of the
    /// parsed XML document.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let document = xot.parse("<hello/>")?;
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn parse(&mut self, xml: &str) -> Result<Node, ParseError> {
        self.parse_with_span_info(xml).map(|(node, _)| node)
    }

    /// Parse a string containing an XML fragment into a document node.
    ///
    /// This is similar to [`Xot::parse``], but it relaxes the well-formedness
    /// requirements. Specifically, it allows text nodes at the top level, and
    /// does not require a document element, and allows multiple elements. In
    /// short, a fragment behaves like an element (without name, attributes or
    /// namespace definitions).
    ///
    /// This is to support
    /// <https://www.w3.org/TR/xpath-datamodel/#DocumentNode> which is more
    /// permissive than standard XML.
    pub fn parse_fragment(&mut self, xml: &str) -> Result<Node, ParseError> {
        self.parse_fragment_with_span_info(xml)
            .map(|(node, _)| node)
    }

    /// Parse bytes containing XML into a node.
    ///
    /// This attempts to decode the data in the bytes into a Rust string
    /// (UTF-8) first, then parses this string.
    ///
    /// If you already have a Rust string, use [`Xot::parse`].
    ///
    /// The returned node is the document node of the parsed XML document.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let document = xot.parse_bytes(b"<hello/>")?;
    ///
    /// # Ok::<(), xot::Error>(())
    /// ```
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    ///
    /// let document = xot.parse_bytes(b"<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?><p>\xe9</p>").unwrap();
    ///
    /// let doc_el = xot.document_element(document)?;
    /// let txt_value = xot.text_content_str(doc_el).unwrap();
    /// assert_eq!(txt_value, "é");
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn parse_bytes(&mut self, bytes: &[u8]) -> Result<Node, ParseError> {
        let xml = decode(bytes, None);
        self.parse(&xml)
    }
}

fn normalize_xml_id(value: &str) -> String {
    // strip both leading and trailing space characters
    let value = value.strip_prefix(' ').unwrap_or(value);
    let value = value.strip_suffix(' ').unwrap_or(value);
    // now take any repeated sequences of the space character ' ' and normalize
    // it to a single space character
    let mut result = String::with_capacity(value.len());
    let mut last_char_space = false;

    for c in value.chars() {
        if c == ' ' {
            if !last_char_space {
                result.push(c);
                last_char_space = true;
            }
        } else {
            result.push(c);
            last_char_space = false;
        }
    }
    result
}
