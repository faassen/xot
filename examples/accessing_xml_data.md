Let's go through this example step by step.

You need to import [Xot](`xot::Xot``). If you want to handle Xot errors, you can
also import the Xot specific [Error](`xot::Error`) enum.

```rust
use xot::{Error, Xot};
```

We then create a `xot` object:

```rust
let mut xot = Xot::new();
```

The `xot` object contains all the XML data in your application. This includes
structural information in the form of nodes, and value information specifying
details about XML data such as elements and text. It also contains information
about element names and namespaces. You can store as many XML trees in a `xot`
as you like.

`xot` needs to be mutable, as we're going to add information to it by parsing:

```rust
let doc = xot.parse("<p>Example</p>")?;
```

We have parsed a document. We get a Xot [Node](`xot::Node`) in return. This is
a lightweight handle to a node in the tree; you can copy it as much as you
like, just like you would with a number.

```rust
let doc_el = xot.document_element(doc)?;
```

We now ask `xot` for the document element for this node; this is the top-level
element. There is always 1 document element in an XML document by definition.
In the XML we just parsed this is the `p` element.

```rust
let txt_node = xot.first_child(doc_el).unwrap();
```

Now we navigate the tree some more: we look for the first child node
of the document element: that's the text node that contains the string
`Example`. This operation returns an option; if there is no first child
(as there are no children) this returns `None`. Since we know there is
a child in this case we can immediately unwrap it.

```rust
let txt_value = xot.text(txt_node).unwrap();
```

Now we obtain the value of the text node. Again this may fail if this
isn't a text node; since we know it is we immediately unwrap it.

```rust
assert_eq!(txt_value.get(), "Example");
```

We access the text contained in the text value using the `get` method.
It's `"Example"` as we expect here.
