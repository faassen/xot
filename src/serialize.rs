use std::io::Write;

use crate::error::Error;
use crate::pretty::Pretty;
use crate::serializer::{gen_outputs, Output, OutputToken, XmlSerializer};
use crate::xmlname::NameStrInfo;
use crate::{output, Value};

use crate::xotdata::{Node, Xot};

/// Pretty output token
///
/// Like [`OutputToken`](`crate::OutputToken`) but with extra information for
/// pretty printing.
pub struct PrettyOutputToken {
    /// indentation level.
    pub indentation: usize,
    /// Whether the token is prefixed by a space character.
    pub space: bool,
    /// The token
    ///
    /// This is a fragment of XML like `"<p"`, `a="A"` or `"</p>"`.
    pub text: String,
    /// Whether the token is suffixed by a newline character.
    pub newline: bool,
}

/// Options to control serialization
#[derive(Debug, Default)]
pub struct SerializeOptions {
    /// Pretty print XML
    pub pretty: bool,
}

/// Configurable serialization
pub struct WithSerializeOptions<'a> {
    xot: &'a Xot,
    options: SerializeOptions,
}

impl<'a> WithSerializeOptions<'a> {
    /// Write node as XML.
    pub fn write(&self, node: Node, w: &mut impl Write) -> Result<(), Error> {
        let outputs = gen_outputs(self.xot, node);
        let mut serializer = XmlSerializer::new(self.xot, node);
        if self.options.pretty {
            serializer.serialize_pretty(w, outputs, vec![])
        } else {
            serializer.serialize(w, outputs)
        }
    }

    /// Write node to XML string.
    pub fn to_string(&self, node: Node) -> Result<String, Error> {
        let mut buf = Vec::new();
        self.write(node, &mut buf)?;
        Ok(String::from_utf8(buf).unwrap())
    }
}

/// ## Serialization
impl Xot {
    /// Write node as XML.
    ///
    /// You can control output options by using [`Xot::with_serialize_options`] first,
    /// and calling `write` on that.
    ///
    /// If there are missing namespace prefixes, this errors. You can
    /// automatically add missing prefixes by invoking
    /// [`Xot::create_missing_prefixes`] before serialization to avoid this
    /// error.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse("<p>Example</p>")?;
    ///
    /// let mut buf = Vec::new();
    /// xot.write(root, &mut buf).unwrap();
    ///
    /// assert_eq!(buf, b"<p>Example</p>");
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn write(&self, node: Node, w: &mut impl Write) -> Result<(), Error> {
        self.with_serialize_options(SerializeOptions::default())
            .write(node, w)
    }

    /// Serialize node as XML string.
    ///
    /// You can control output options by using [`Xot::with_serialize_options`] first,
    /// and calling `to_string` on that.
    ///
    /// If there are missing namespace prefixes, this errors. You can automatically
    /// add missing prefixes by invoking [`Xot::create_missing_prefixes`] before
    /// serialization to avoid this error.
    ///
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse("<p>Example</p>")?;
    ///
    /// let buf = xot.to_string(root)?;
    ///
    /// assert_eq!(buf, "<p>Example</p>");
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn to_string(&self, node: Node) -> Result<String, Error> {
        self.with_serialize_options(SerializeOptions::default())
            .to_string(node)
    }

    /// Serialize to XML, with options.
    ///
    /// With the default parameters:
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse("<a><b/></a>")?;
    ///
    /// let xml = xot.serialize_xml(Default::default(), root)?;
    /// assert_eq!(xml, "<a><b/></a>");
    /// # Ok::<(), xot::Error>(())
    /// ```
    ///
    /// With an XML declaration:
    ///
    /// ```rust
    /// use xot::{Xot, output};
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse("<a><b/></a>")?;
    ///
    /// let xml = xot.serialize_xml(output::xml::Parameters {
    ///     declaration: Some(Default::default()),
    ///     ..Default::default()
    /// }, root)?;
    /// assert_eq!(xml, "<?xml version=\"1.0\"?>\n<a><b/></a>");
    /// # Ok::<(), xot::Error>(())
    /// ```
    ///
    /// XML declaration with an encoding declaration (does not affect output encoding):
    ///
    /// ```rust
    /// use xot::{Xot, output};
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse("<a><b/></a>")?;
    ///
    /// let xml = xot.serialize_xml(output::xml::Parameters {
    ///     declaration: Some(output::xml::Declaration {
    ///         encoding: Some("UTF-8".to_string()),
    ///         ..Default::default()
    ///     }),
    ///     ..Default::default()
    /// }, root)?;
    /// assert_eq!(xml, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<a><b/></a>");
    /// # Ok::<(), xot::Error>(())
    /// ```
    ///
    /// Pretty print XML:
    ///
    /// ```rust
    /// use xot::{Xot, output};
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse("<a><b/></a>")?;
    ///
    /// let xml = xot.serialize_xml(output::xml::Parameters {
    ///     indentation: Some(Default::default()),
    ///     ..Default::default()
    /// }, root)?;
    /// assert_eq!(xml, "<a>\n  <b/>\n</a>\n");
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn serialize_xml(
        &self,
        parameters: output::xml::Parameters,
        node: Node,
    ) -> Result<String, Error> {
        let mut buf = Vec::new();
        if let Some(declaration) = parameters.declaration {
            declaration.serialize(&mut buf)?;
        }
        if let Some(doctype) = parameters.doctype {
            // if we are in a document node, we look for the document_element,
            // otherwise we take the current element, if possible
            let node = match self.value(node) {
                Value::Document => self.document_element(node)?,
                Value::Element(_) => node,
                _ => return Err(Error::NotElement(node)),
            };
            // now take the full name of the element; we can unwrap as we
            // know it's an element now
            let name = self.node_name_ref(node)?.unwrap();
            let name = name.full_name();
            doctype.serialize(name.as_ref(), &mut buf)?;
        }
        let outputs = gen_outputs(self, node);
        let mut serializer = XmlSerializer::new(self, node);
        if let Some(indentation) = parameters.indentation {
            serializer.serialize_pretty(&mut buf, outputs, indentation.suppress.clone())?;
        } else {
            serializer.serialize(&mut buf, outputs)?;
        }
        Ok(String::from_utf8(buf).unwrap())
    }

    /// Control XML serialization
    ///
    /// You can control the serialization before invoking [`WithSerializeOptions::write`] or
    /// [`WithSerializeOptions::to_string`] by passing in options.
    ///
    /// ```rust
    /// use xot::{Xot, SerializeOptions};
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse("<a><b/></a>")?;
    ///
    /// let buf = xot.with_serialize_options(SerializeOptions { pretty: true, ..SerializeOptions::default() }).to_string(root)?;
    ///
    /// assert_eq!(buf, "<a>\n  <b/>\n</a>\n");
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn with_serialize_options(&self, options: SerializeOptions) -> WithSerializeOptions {
        WithSerializeOptions { xot: self, options }
    }

    /// Serialize node into outputs.
    ///
    /// This creates an iterator of `(Node, Output)` tokens. These can then be
    /// processed externally, for instance with a custom parser.
    ///
    /// You can use [`Xot::parse_with_span_info`] to get access to source span
    /// information in a [`crate::SpanInfo`]. This can be handy to display
    /// parser error information (as long as you don't mutate the Xot
    /// document).
    ///
    /// Why do the round-trip through Xot instead of using an XML parser like
    /// `xmlparser`, `xml_rs` or `quick_xml`, which will be more efficient? By
    /// using Xot you can guarantee that the XML is well-formed, entities and
    /// namespaces have been expanded, and you have access to Xot names using
    /// familiar Xot APIs.
    pub fn outputs(&self, node: Node) -> impl Iterator<Item = (Node, Output)> {
        gen_outputs(self, node)
    }

    /// Serialize node into outputs and tokens.
    ///
    /// This creates an iterator that represents the serialized XML. You
    /// can use this to write custom renderers that serialize the XML in
    /// a different way, for instance with inline styling.
    pub fn tokens(&self, node: Node) -> impl Iterator<Item = (Node, Output, OutputToken)> + '_ {
        let outputs = gen_outputs(self, node);
        let mut serializer = XmlSerializer::new(self, node);
        outputs.map(move |(node, output)| {
            let rendered = serializer.render_output(node, &output).unwrap();
            (node, output, rendered)
        })
    }

    /// Serialize node into outputs and pretty printed tokens.
    ///
    /// This creates an iterator that represents the serialized XML. You
    /// can use this to write custom renderers that serialize the XML in
    /// a different way, for instance with inline styling.
    pub fn pretty_tokens(
        &self,
        node: Node,
    ) -> impl Iterator<Item = (Node, Output, PrettyOutputToken)> + '_ {
        let outputs = gen_outputs(self, node);
        let mut serializer = XmlSerializer::new(self, node);
        let mut pretty = Pretty::new(self, vec![]);
        outputs.map(move |(node, output)| {
            let (indentation, newline) = pretty.prettify(node, &output);
            let rendered = serializer.render_output(node, &output).unwrap();
            (
                node,
                output,
                PrettyOutputToken {
                    text: rendered.text,
                    space: rendered.space,
                    indentation,
                    newline,
                },
            )
        })
    }
}
