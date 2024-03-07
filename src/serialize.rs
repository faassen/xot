use std::io::Write;

use crate::error::Error;
use crate::output::{
    gen_outputs, Html5Elements, Html5Serializer, Output, OutputToken, XmlSerializer,
};
use crate::output::{NoopNormalizer, Normalizer};
use crate::output::{Pretty, PrettyOutputToken};
use crate::xmlname::NameStrInfo;
use crate::{output, NameId, Value};

use crate::xotdata::{Node, Xot};

pub struct Html5<'a> {
    xot: &'a Xot,
    html5_elements: Html5Elements,
}

impl<'a> Html5<'a> {
    fn new(xot: &'a mut Xot) -> Self {
        let html5_elements = Html5Elements::new(xot);
        Html5 {
            xot,
            html5_elements,
        }
    }

    /// Serialize to HTML5, default settings.
    pub fn to_string(&self, node: Node) -> Result<String, Error> {
        self.serialize_string(Default::default(), node)
    }

    /// Write to HTML5, default settings.
    pub fn write(&self, node: Node, w: &mut impl Write) -> Result<(), Error> {
        self.serialize_write(Default::default(), node, w)
    }

    /// Serialize to HTML 5 via a [`Write`], with options.
    pub fn serialize_write(
        &self,
        parameters: output::html5::Parameters,
        node: Node,
        w: &mut impl Write,
    ) -> Result<(), Error> {
        self.serialize_write_with_normalizer(parameters, node, w, NoopNormalizer)
    }

    /// Serialize to HTML 5 string.
    pub fn serialize_string(
        &self,
        parameters: output::html5::Parameters,
        node: Node,
    ) -> Result<String, Error> {
        self.serialize_string_with_normalizer(parameters, node, NoopNormalizer)
    }

    /// Serialize to HTML 5 string, with normalizer.
    pub fn serialize_string_with_normalizer<N: Normalizer>(
        &self,
        parameters: output::html5::Parameters,
        node: Node,
        normalizer: N,
    ) -> Result<String, Error> {
        let mut buf = Vec::new();
        self.serialize_write_with_normalizer(parameters, node, &mut buf, normalizer)?;
        Ok(String::from_utf8(buf).unwrap())
    }

    /// Write HTML 5 with a normalizer for text and attribute values.
    pub fn serialize_write_with_normalizer<N: Normalizer>(
        &self,
        parameters: output::html5::Parameters,
        node: Node,
        w: &mut impl Write,
        normalizer: N,
    ) -> Result<(), Error> {
        w.write_all(b"<!DOCTYPE html>").unwrap();
        let outputs = gen_outputs(self.xot, node);
        let mut serializer = Html5Serializer::new(
            self.xot,
            &self.html5_elements,
            node,
            &parameters.cdata_section_elements,
            normalizer,
        );
        if let Some(indentation) = parameters.indentation {
            serializer.serialize_pretty(w, outputs, &indentation.suppress)?;
        } else {
            serializer.serialize(w, outputs)?;
        }
        Ok(())
    }
}

/// ## Serialization
impl Xot {
    /// Write node as XML.
    ///
    /// This uses the default serialization parameters: no XML declaration, no
    /// doctype, no control over pretty printing or CDATA.
    ///
    /// To serialize with this control use [`Xot::serialize_xml_write`].
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
        self.serialize_xml_write(Default::default(), node, w)
    }

    /// Serialize node as XML string.
    ///
    /// This uses the default serialization parameters: no XML declaration, no
    /// doctype, no control over pretty printing or CDATA.
    ///
    /// To serialize with this control use [`Xot::serialize_xml_string`].
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
        self.serialize_xml_string(Default::default(), node)
    }

    /// Serialize to XML, with options.
    ///
    /// Note that if you don't need string output and have a writer available,
    /// a more efficient option is to use [`Xot::serialize_xml_write`].
    ///
    /// If there are missing namespace prefixes, this errors. You can automatically
    /// add missing prefixes by invoking [`Xot::create_missing_prefixes`] before
    /// serialization to avoid this error.
    ///
    /// With the default parameters:
    /// ```rust
    /// use xot::Xot;
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse("<a><b/></a>")?;
    ///
    /// let xml = xot.serialize_xml_string(Default::default(), root)?;
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
    /// let xml = xot.serialize_xml_string(output::xml::Parameters {
    ///     declaration: Some(Default::default()),
    ///     ..Default::default()
    /// }, root)?;
    /// assert_eq!(xml, "<?xml version=\"1.0\"?>\n<a><b/></a>");
    /// # Ok::<(), xot::Error>(())
    /// ```
    ///
    /// XML declaration with an encoding declaration (does not affect output
    /// encoding):
    ///
    /// ```rust
    /// use xot::{Xot, output};
    ///
    /// let mut xot = Xot::new();
    /// let root = xot.parse("<a><b/></a>")?;
    ///
    /// let xml = xot.serialize_xml_string(output::xml::Parameters {
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
    /// let xml = xot.serialize_xml_string(output::xml::Parameters {
    ///     indentation: Some(Default::default()),
    ///     ..Default::default()
    /// }, root)?;
    /// assert_eq!(xml, "<a>\n  <b/>\n</a>\n");
    /// # Ok::<(), xot::Error>(())
    /// ```
    pub fn serialize_xml_string(
        &self,
        parameters: output::xml::Parameters,
        node: Node,
    ) -> Result<String, Error> {
        self.serialize_xml_string_with_normalizer(parameters, node, NoopNormalizer)
    }

    /// Serialize a string using a normalizer for any text and attribute values.
    ///
    /// If you enable the `icu` feature then support for [icu normalizers](https://docs.rs/icu/latest/icu/normalizer/index.html)
    /// is provided.
    #[cfg_attr(
        feature = "icu",
        doc = r##"
You can pass in an icu normalizer like this:

```rust
use xot::{Xot, output};
use icu::normalizer::ComposingNormalizer;
let normalizer = ComposingNormalizer::new_nfc();
// example taken from https://www.unicode.org/reports/tr15/ figure 5, second example
let xml = "<doc>\u{1E0B}\u{0323}</doc>";
let mut xot = Xot::new();
let root = xot.parse(xml).unwrap();
let s = xot.serialize_xml_string_with_normalizer(output::xml::Parameters::default(), root, normalizer).unwrap();
assert_eq!(s, "<doc>\u{1E0D}\u{0307}</doc>");
```
"##
    )]
    pub fn serialize_xml_string_with_normalizer<N: Normalizer>(
        &self,
        parameters: output::xml::Parameters,
        node: Node,
        normalizer: N,
    ) -> Result<String, Error> {
        let mut buf = Vec::new();
        self.serialize_xml_write_with_normalizer(parameters, node, &mut buf, normalizer)?;
        Ok(String::from_utf8(buf).unwrap())
    }

    /// Serialize to XML via a [`Write`], with options.
    ///
    /// This is like [`Xot::serialize_xml_string`] but writes to a [`Write`]. This
    /// is more efficient if you want to write directly to a file, for
    /// instance, as no string needs to be created in memory.
    pub fn serialize_xml_write(
        &self,
        parameters: output::xml::Parameters,
        node: Node,
        w: &mut impl Write,
    ) -> Result<(), Error> {
        self.serialize_xml_write_with_normalizer(parameters, node, w, NoopNormalizer)
    }

    /// Write XML with a normalizer for text and attribute values.
    ///
    /// See [`Xot::serialize_xml_string_with_normalizer`] for more information.
    pub fn serialize_xml_write_with_normalizer<N: Normalizer>(
        &self,
        parameters: output::xml::Parameters,
        node: Node,
        w: &mut impl Write,
        normalizer: N,
    ) -> Result<(), Error> {
        if let Some(declaration) = parameters.declaration {
            declaration.serialize(w)?;
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
            doctype.serialize(name.as_ref(), w)?;
        }
        let outputs = gen_outputs(self, node);
        let mut serializer =
            XmlSerializer::new(self, node, &parameters.cdata_section_elements, normalizer);
        if let Some(indentation) = parameters.indentation {
            serializer.serialize_pretty(w, outputs, &indentation.suppress)?;
        } else {
            serializer.serialize(w, outputs)?;
        }
        Ok(())
    }

    /// Get HTML 5 serialization API.
    ///
    /// This is a mutable calls as it needs to create a lot of new HTML names
    /// first.
    ///
    /// If you need to generate multiple HTML 5 serializations, it's slightly
    /// more efficient not to re-create this each time.
    pub fn html5(&mut self) -> Html5 {
        Html5::new(self)
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
    ///
    /// You can include a list of element names that should be serialized as a
    /// CDATA section.
    /// You can also pass in a normalizer; if you don't care about normalization, use
    // [`output::xml::NoopNormalizer`].
    pub fn tokens<'a, N: Normalizer + 'a>(
        &'a self,
        node: Node,
        cdata_section_elements: &'a [NameId],
        normalizer: N,
    ) -> impl Iterator<Item = (Node, Output, OutputToken)> + 'a {
        let outputs = gen_outputs(self, node);
        let mut serializer = XmlSerializer::new(self, node, cdata_section_elements, normalizer);
        outputs.map(move |(node, output)| {
            let rendered = serializer.render_output(node, &output).unwrap();
            (node, output, rendered)
        })
    }

    /// Serialize node into outputs and pretty printed tokens.
    ///
    /// This creates an iterator that represents the serialized XML. You can
    /// use this to write custom renderers that serialize the XML in a
    /// different way, for instance with inline styling.
    ///
    /// You can include a list of elements names that are excluded from
    /// indentation, and a list of elements that should be serialized as a
    /// CDATA section.
    ///
    /// You can also pass in a normalizer; if you don't care about normalization, use
    // [`output::xml::NoopNormalizer`].
    pub fn pretty_tokens<'a, N: Normalizer + 'a>(
        &'a self,
        node: Node,
        suppress_elements: &'a [NameId],
        cdata_section_elements: &'a [NameId],
        normalizer: N,
    ) -> impl Iterator<Item = (Node, Output, PrettyOutputToken)> + 'a {
        let outputs = gen_outputs(self, node);
        let mut serializer = XmlSerializer::new(self, node, cdata_section_elements, normalizer);
        let mut pretty = Pretty::new(
            self,
            |name| suppress_elements.contains(&name),
            |_name| false,
        );
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
