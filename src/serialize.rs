use std::io::Write;

use crate::pretty::PrettyWriter;
use crate::serializer::{Serializer, SerializerWriter, XmlSerializerWriter};

use crate::error::Error;
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

impl<'a> WithSerializeOptions<'a> {
    /// Write node as XML.
    pub fn write(&self, node: Node, w: &mut impl Write) -> Result<(), Error> {
        if self.options.pretty {
            let mut serializer_writer = PrettyWriter::new(self.xot, w);
            let mut serializer = Serializer::new(self.xot, &mut serializer_writer);
            serializer.serialize_node(node)
        } else {
            let mut serializer_writer = XmlSerializerWriter::new(self.xot, w);
            let mut serializer = Serializer::new(self.xot, &mut serializer_writer);
            serializer.serialize_node(node)
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

    /// Serialize node with a custom serializer writer.
    ///
    /// This is an advanced method that allows customisation of the XML writing.
    ///
    /// If there are missing namespace prefixes, this errors. You can automatically
    /// add missing prefixes by invoking [`Xot::create_missing_prefixes`] before
    /// serialization to avoid this error.
    pub fn serialize_with_writer(
        &self,
        node: Node,
        serializer_writer: &mut impl SerializerWriter,
    ) -> Result<(), Error> {
        let mut serializer = Serializer::new(self, serializer_writer);
        serializer.serialize_node(node)
    }
}
