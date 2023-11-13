use next_gen::prelude::*;
use std::io::Write;

use crate::error::Error;
use crate::pretty::Pretty;
use crate::serializer::{gen_outputs, Output, OutputToken, XmlSerializer};

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
        mk_gen!(let outputs = gen_outputs(self.xot, node));
        let mut serializer = XmlSerializer::new(self.xot, node);
        if self.options.pretty {
            serializer.serialize_pretty(w, outputs)
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
        mk_gen!(let outputs = box gen_outputs(self, node));
        outputs
    }

    /// Serialize node into outputs and tokens.
    ///
    /// This creates an iterator that represents the serialized XML. You
    /// can use this to write custom renderers that serialize the XML in
    /// a different way, for instance with inline styling.
    pub fn tokens(&self, node: Node) -> impl Iterator<Item = (Node, Output, OutputToken)> + '_ {
        mk_gen!(let outputs = box gen_outputs(self, node));
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
        mk_gen!(let outputs = box gen_outputs(self, node));
        let mut serializer = XmlSerializer::new(self, node);
        let mut pretty = Pretty::new(self);
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
