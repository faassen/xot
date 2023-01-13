use next_gen::prelude::*;
use std::io::Write;

use crate::error::Error;
use crate::pretty::Pretty;
use crate::serializer::{gen_outputs, Output, Token, XmlSerializer};

use crate::xotdata::{Node, Xot};

/// Options to control serialization
#[derive(Debug, Default)]
pub struct SerializeOptions {
    /// Pretty print XML
    pub pretty: bool,
}

pub struct WithSerializeOptions<'a> {
    xot: &'a Xot<'a>,
    options: SerializeOptions,
}

pub struct PrettyToken {
    pub indentation: usize,
    pub space: bool,
    pub text: String,
    pub newline: bool,
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
impl<'a> Xot<'a> {
    /// Write node as XML.
    ///
    /// You can control output options by using [`Xot::serialize_options`] first,
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
        self.serialize_options(SerializeOptions::default())
            .write(node, w)
    }

    /// Serialize node as XML string.
    ///
    /// You can control output options by using [`Xot::serialize_options`] first,
    /// and calling `serialize` on that.
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
        self.serialize_options(SerializeOptions::default())
            .to_string(node)
    }

    /// Control XML serialization
    ///
    /// You can control the serialization before invoking [`WithSerializationOptions::write`] or
    /// [`WithSerializationOptions::to_string`] by passing in options.
    ///
    /// ```rust
    /// use xot::{Xot, SerializeOptions};
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse("<a><b/></a>")?;
    ///
    /// let buf = xot.serialize_options(SerializeOptions { pretty: true, ..SerializeOptions::default() }).to_string(root)?;
    ///
    /// assert_eq!(buf, "<a>\n  <b/>\n</a>\n");
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn serialize_options(&self, options: SerializeOptions) -> WithSerializeOptions {
        WithSerializeOptions { xot: self, options }
    }

    pub fn tokens(&'a self, node: Node) -> impl Iterator<Item = (Node, Output<'a>, Token)> + '_ {
        mk_gen!(let outputs = box gen_outputs(self, node));
        let mut serializer = XmlSerializer::new(self, node);
        outputs.map(move |(node, output)| {
            let rendered = serializer.render_output(node, &output).unwrap();
            (node, output, rendered)
        })
    }

    pub fn pretty_tokens(
        &'a self,
        node: Node,
    ) -> impl Iterator<Item = (Node, Output<'a>, PrettyToken)> + '_ {
        mk_gen!(let outputs = box gen_outputs(self, node));
        let mut serializer = XmlSerializer::new(self, node);
        let mut pretty = Pretty::new(self);
        outputs.map(move |(node, output)| {
            let (indentation, newline) = pretty.prettify(node, &output);
            let rendered = serializer.render_output(node, &output).unwrap();
            (
                node,
                output,
                PrettyToken {
                    text: rendered.text,
                    space: rendered.space,
                    indentation,
                    newline,
                },
            )
        })
    }
}
