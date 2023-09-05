# Xot - XML Object Tree

[![Crates.io](https://img.shields.io/crates/v/xot.svg)](https://crates.io/crates/xot)
[![Documentation](https://docs.rs/xot/badge.svg)](https://docs.rs/xot)

Xot is an XML library for Rust. With Xot you can load XML data into an object
tree which you can then manipulate, and serialize back out to XML.

[API Documentation](https://docs.rs/xot)

## Features

- Extensive XML tree access and manipulation API.

- Move nodes out of trees, into trees, into other trees with ease.

- Access to the entire XML structure, including comments and processing
  instructions.

- Compact storage of XML data, with efficient traversal.

- Parse XML into a tree, and serialize back to XML.

- Pretty-printing. Removal of non-significant whitespace.

## Support

Xot has well-rounded XML support. Supported are namespaces for both elements
and attributes. You can also get comments and processing instructions.

Xot exposes the entire XML structure for access and programmatic manipulation.

## Limitations

There are the following limitations:

- DTDs are not supported.

## Why another XML library for Rust?

Use Xot if you need to be able to access XML as a tree, mutate it where
necessary, and serialize it back out again. Various Rust XML tree libraries
exist but none seem particularly interested in a complete tree manipulation
API, so Xot aims to fill that gap.

If you need to access XML as a tree, very high performance, and do not need
tree manipulation or serialization at all, I recommend
[roxmltree](https://github.com/RazrFalcon/roxmltree).

For extensive features such as Xpath as well as tree manipulation, you can use
[libxml](https://github.com/KWARC/rust-libxml) which wraps the libxml2 C
library. Unfortunately its tree access facilities aren't entirely complete, and
I've experienced odd errors and even crashes when using it; this was part of
the reason I created Xot.

If you need to process big XML data with high performance, you may not want to
use a tree library and instead use one of the various XML parser/serialization
libraries.

## Implementation notes

Xot underneath uses the
[indextree](https://docs.rs/indextree/latest/indextree/) crate. Xot completely
wraps the indextree functionality but the various operations it allows are
mostly literally taken from indextree.
