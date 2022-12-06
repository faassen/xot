# Xot - XML Object Tree

Xot is an XML library for Rust. With Xot you can load XML data into an object
tree which you can then manipulate, and serialize back out to XML.

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
library. Unfortunately its tree manipulation facilities entirely complete, and
I've experienced odd errors and even crashes when using it; this was part
of the reason I created Xot.

If you need to process big XML data with high performance, you may not want to
use a tree library and instead use one of the various parser/serialization
libraries.
