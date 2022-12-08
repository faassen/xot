# Xot - XML Object Tree

Xot is an XML library for Rust. With Xot you can load XML data into an object
tree which you can then manipulate, and serialize back out to XML.

## Support

Xot has well-rounded XML support. Supported are namespaces for both elements
and attributes. You can also get comments and processing instructions. Both
documents as well as XML fragments are supported.

Xot exposes the entire XML structure for access and programmatic manipulation.

## Limitations

There are two limitations:

- DTDs are not supported.

- Only the UTF-8 encoding is supported.

## Xot approach

Xot exposes a single `Xot` struct that you use to access, create and
manipulate all your XML data. Multiple XML trees can exist in an `Xot`
struct at the same time, and you're free to move nodes between these trees.

The way to access and manipulate XML tree structure in Xot is through the
`Node`. You can use various functions on `Xot` in order to access and
manipulate information about nodes.

To access and manipulate XML specific data, you use the `Value` for a node.
This is an enum that's either an `Element`, `Text`, `Root`, `Comment` or
`ProcessingInstruction`.

XML names and namespaces in Xot are consolidated into ids. In order to
construct or compare an element, you first need to get hold of a name.

## Why another XML library for Rust?

Use Xot if you need to be able to access XML as a tree, mutate it where
necessary, and serialize it back out again. Various XML tree libraries exist
but none seem particularly interested in a complete tree manipulation API.

Xot doesn't aim to be the fastest XML library out there, though it being Rust
its performance should still be pretty good.

If you need to access XML as a tree, very high performance, and do not need
tree manipulation or serialization at all, I recommend
[roxmltree](https://github.com/RazrFalcon/roxmltree).

For features such as tree manipulation and Xpath, you can use
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
