<!-- next-header -->

## [Unreleased] - ReleaseDate

### Breaking changes

- Previously a `>` character in text was serialized as itself not as `&gt;`, as
  this is allowed by the spec (except `]]>`, see the bug fix below). Since this
  can be rather surprising, the default is now to always serialize `>` as
  `&gt;`.

- `xot.tokens` and `xot.pretty_tokens` now take a
  `xot::output::TokenSerializeParameters` object that can be used to control
  token generation, as opposed to manually passing in `cdata_section_elements`.

### Features added

- Added `xot.get_element_name`, `xot.set_element_name` convenience APIs.

- Added `xot.get_attribute`,`xot.set_attribute`,
  `xot.delete_attribute`, and `xot.get_namespace`, `xot.set_namespace`,
  `xot.delete_namespace` convenience APIs.

- Added convenience `to_hashmap` on attributes and namespaces accessors.

### Bugs fixed

- `]]>` if seen outside of a CDATA section delimiter should always be rendered
  as `]]&gt;`, and it wasn't before. Now it is even if unescaped gts are
  configured.

- Fixed a bug with `xot.prepend`; it didn't work correctly in the presence of
  namespace and attribute nodes.

- `xot.clone_with_prefixes` crashed if you tried to clone a non-element node.

- `is_empty` for `xot.attributes()` and `xot.namespaces()` gave the wrong output.

- `unwrap_element` had a bug where the element contained namespaces or
  attributes.

## [0.23.0] - 2024-03-22

### Breaking changes

- Created a new API with much more control of serialization:
  `Xot::serialize_xml_string` and `Xot::serialize_xml_write`, which are like
  `Xot::to_string` and `Xot::write`, but allow fine-grained control of
  serialization options, including pretty printing.

- Removed `Xot::with_serialize_options`, `SerializeOptions`, `WithSerializeOptions` in    
  favor of the new APIs.

- The low-level `Xot::tokens` API has additional parameters, a list of element
  names that should be serialized as cdata sections. Putting in an empty vec is
  fine if you don't want any. You can also pass in a normalizer (if you don't
  care, use the `xot::output::NoopNormalizer`).

- The low-level `Xot::pretty_tokens` API has additional parameters: a list of
  elements that should be included from indentation and a list of that should
  be serialized as cdata sections. You can also pass in a normalizer (if you don't
  care, use the `xot::output::NoopNormalizer`).

- `xot::Output`, `xot::OutputToken`, `xot::PrettyOutputToken` are now
  importable from `xot::output` instead of from `xot` directly.

## Changes

### Rust 1.77 compatibility

There was a problem compiling this with Rust 1.77 which has since been fixed.

### New `output::xml::Parameters` API to control XML serialization

- You can cause an XML declaration to be output.

- You can cause a doctype declaration to be output.

- You can suppress specific element names from being pretty-printed.

- You can cause specific element names content to be presented as CDATA
  sections.

- You can cause text content to be unicode normalized. This requires the new
  feature `icu` feature to be enabled. See
  ``Xot::serialize_xml_string_with_normalizer` for more information.

### New API (`Xot::html5`) and `output::html5::Parameters` API to control HTML 5 serialization

- You can output XML as HTML 5 now.

- Indentation rules know about HTML inline elements.

## [0.22.0] - 2024-02-20

### Breaking changes

- `MissingPrefix` now contains the namespace for which the prefix cannot be
  found as a string, not as a `NamespaceId`.

### Changes

- Introduce `xmlname` module which supports advanced use cases surrounding the
  creation and access to XML names. It also offers more convenient creation and
  access of names as opposed to the gaggle of methods that exist on `Xot`.

- The various APIs that take name ids have been modified to accept some
  `xmlname` names (`xmlname::Create`, `xmlname::Ref`) as well. The
  documentation for `new_element` has an example.

- Added `Xot::add_namespace` to create a namespace node based on an
  `xmlname::Namespace`.

- Added `Xot::name_ref` and `Xot::node_name_ref` to get `xmlname::Ref` access to
  a node.

### Changes

Move `id` related functionality into `id` submodule.

## [0.21.0] - 2024-02-15

### Breaking changes

Warning: this release includes many major API breaking changes!

#### Attribute and namespace API changes

The way attributes and namespace prefixes are handled has changed entirely in
this release. Instead of maintaining this information on the `Element` value
(as a vecmap), the information is now entirely handled in-tree. This means that
attributes and namespace prefixes are now addressable as nodes, with as their
parent the element node.

This means some significant breaking API changes:

* The `Element` value does not maintain attributes and namespace prefixes
  anymore. The only thing it has is a `name()` accessor, and a way to
  set the name with `set_name()`.

* The `Value` enum gains two new entries, `Attribute` and `Namespace`.
  Similarly, `ValueType` gains these as well.

* To access attributes, use `xot.attributes(node)`. This returns a hashmap-like
  struct (`NodeMap`). with as key the name id of the attribute, and as value as
  string, that lets you access attributes using `get`, iterate over them, etc.
  `NodeMap`, like `VecMap`, maintains order information and access time is
  linear, not constant time like a real hashmap.

* To access namespaces, use `xot.namespaces(node)`. This returns a hashmap-like
  struct with as key the prefix id, and as value the namespace id. The old
  `xot.namespaces` has been renamed to `xot.inherited_prefixes`. You can 
  also access this information as a `Prefixes` map, using `xot.prefixes(node)`.

* `xot::Prefixes` is now a real std hashmap, not a vecmap or the new nodemap,
   and thus does not retain order information. It is returned from specific
   APIs such as `xot.prefixes(node)` and `xot.inherited_prefixes()`.

* You can also update and add attributes and namespaces using `xot.
  attributes_mut()` and `xot.namespaces_mut()`, respectively. Since these hold
  a mutable reference to the tree, lifetime issues makes them not so convenient
  for accessing the information.

* While `namespaces` and `attributes` return hashmap-like APIs, this API is at
  present not as complete as attribute and prefixes information previously
  returned as provided by `vecmap-rs`.

* Comparing two elements used to be possible through
  `element.compare_ignore_attributes`. This has now been replaced by
  `xot.shallow_equal_ignore_attributes`, which can also compare non-element
  nodes.

#### Attribute and namespace nodes

The previously discussed `xot.attributes[_mut]` and `xot.namespaces[_mut]` are the most convenient APIs for most use cases.

But in some advanced use cases you want to be able to directly access and manipulate attribute and namespace nodes.

This is done with a new set of APIs. These all have the postfix `_node` in them
to remind you that you are doing explicit node manipulation:

* To create a new namespace node: `xot.new_namespace_node()`.

* To create a new attribute node: `xot.new_attribute_node()`.

* To append a namespace node: `xot.append_namespace_node()`.

* To append an attribute node: `xot.append_attribute_node()`.

* To append any node, including namespace and attribute nodes, use
  `xot.any_append()` 

* To check whether something is a namespace node: `xot.is_namespace_node()`.

* To check whether something is an attribute node: `xot.is_attribute_node()`.

* To access or manipulate namespace information on a node:
  `xot.namespace_node` and `xot.namespace_node_mut`.

* To access or manipulate attribute information on a node:
  `xot.attribute_node` and `xot.attribute_node_mut`.

* To access an individual namespace node you can also use `get_node` on
  `Attributes` and `Namespaces`, so for instance
  `xot.namespaces(node).get_node(name)`.

* To access all attributes nodes as an iterator, use `xot.attribute_nodes`.

#### Serializer `Output` enum

The serializer `Output` enum has been simplified:

* Any reference to `Element` has become a copy of `Element`, as element is
  basically just a name and now copy.

* The `Output::Prefix`, `Output::Attribute` `Output::StartTagClose` entries
  don't include the element anymore.

* The enum entries `Output::PrefixesFinished` and `Output::AttributesFinished`
  have been removed.

#### Rename compare APIs to equal

Renamed `compare` to `deep_equal`, `advanced_compare` to
`advanced_deep_equal`,`compare_children` to `deep_equal_children`.

Introduced `xot.shallow_compare` and
`xot.shallow_equal_elements_ignore_attributes` to replace removed
`compare_ignore_attributes` on `Element`.

#### Root renamed to Document

To be more in line with XML naming conventions, renamed the `Root` value to the
`Document` value. So:

* `Value::Root` -> `Value::Document`

* `ValueType:Root` -> `ValueType::Document`

* `Error::NotRoot` -> `Error::NotDocument`

* `xot.new_root_unconnected` -> `xot.new_document`

* `xot.new_root` -> `xot.new_document_with_element`

* `xot.is_root` -> `xot.is_document`

* `Fixed::Root` -> `Fixed::Document`

* `Fixed::RootContent` -> `Fixed::DocumentContent`

* `xot.is_under_root` -> `xot.has_document_parent`

#### Processing instruction target

Processing instruction `target` is now a `xot::NameId`, not a string.

### Other changes

#### New APIs

- Introduced `xot.full_name`, `xot.local_name_str` and `xot.uri_str` to get
various aspects of a node name. 

- Also added `xot.node_name` function to retrieve name id of element, attribute
or processing instruction target, according to XPath rules.

- Added `xot.string_value` accessor, which gives the string value of a node 
  according to XPath rules.

- New `xot.axis` method which lets you do traversal according to axis,
following XPath.

- Added `xot.deep_equal_xpath` which compares nodes as defined by XPath.

- Added `xot.all_descendants` iterator which includes name and attribute nodes.

- Added `xot.all_traverse` iterator which includes name and attribute nodes.

#### Dependencies

- Eliminated `vecmap-rs` dependency.

- Internally, `next-gen` has been replaced with `genawaiter` to support
  generators. Should not be visible to the outside, but changes the
  dependencies.

## [0.20.0] - 2024-02-08

### Changes

- Added `child_index` method to get the index of a child in a parent.

## [0.19.0] - 2024-01-15

### Changes

- Add `Debug` support for `Xot` struct.

- Add `xot.new_root_unconnected` to create a root node without any document
  element attached.

- Now always expose the `xot::fixed` API, even if the proptest feature is
  disabled. It turns out to be handy to be able to create fixed content and
  then transform it into Xot data for other use cases as well. The details of
  the `fixed` API have also been reworked.

## [0.18.0] - 2024-01-08

### Changes

- Changed representation of attributes and prefixes from
  `std::collections::BTreeMap` to use `VecMap` from `vecmap-rs`. This has the
  benefit of retaining the original definition order. Note that `vecmap-rs` is
  a different implementation than `vector-map`, which Xot used until version
  0.14. `vecmap-rs` offers an ordering guarantee, while `vector-map` doesn't.

  This could cause some breaking changes if there are differences in the map
  API, or if you used the `BTreeMap` type directly instead of Xot's
  `Attributes` and `Prefixes` type.

## [0.17.0] - 2023-11-13

### Changes

- Add `xot.parse_with_spans()` which besides parsing the tree also returns a
  `xot::SpanInfo` which maintains span information. The span information
  becomes invalid as soon as you mutate a Xot document.

- Add `xot.outputs` to get serialization output tokens out without rendering
  them. This can be useful if you want to process these tokens with an external
  parser.

## [0.16.0] - 2023-09-07

### Changes

- Add `xot.following()` function. This returns all nodes that follow a node in
  document order, not including the node itself or any of its ancestors or
  descendants.

- Add `xot.preceding()` function. This returns all nodes that precede a node in
  document order, not including the node itself or any of its ancestors.

## [0.15.0] - 2023-09-06

### Changes

- Add `xot.namespace_for_prefix()` function to look up namespace in the context
  of a node.

- Add `xot.namespaces()` function that returns an iterator that iterates over
  non-overridden `(prefix, namespace)`` pairs in the scope of a node.

- Add `xot.advanced_compare()` function that lets you compare two nodes with a
  node filter and with a custom text/attribute value comparison.

## [0.14.0] - 2023-08-24

### Changes

- Support for XML documents with other encodings than just UTF-8 and US-ASCII.

- Do not use `VecMap` anymore to store attributes and namespace prefixes.
  Instead store attributes in a `std::collections::BTreeMap`, which should
  behave similarily for small amounts of attributes.

  This is a potentially breaking change when accessing `prefixes` or
  `attributes` on a XML value, but the map API compatibility should in fact be
  improved.

  This can also change the order in which attributes or namespace declarations
  are serialized, which is not a semantic change of XML but may break certain
  tests.

## [0.13.5] - 2023-06-29

### Fixed

- There was a stray debugging print that caused output when parsing CDATA.
  This has been removed.

## [0.13.4] - 2023-06-29

### Fixed

- `deduplicate_namespaces` had a bug. Attributes, unlike elements, can only be
  in a namespace if they have an explicit prefix; there is no defaulting. This
  means that a prefix definition for a duplicate namespace can only safely be
  removed if this prefix doesn't overlap with a default namespace, or if the
  prefix is not in use by any attribute. We now check for this.

## [0.13.3] - 2023-06-26

### Fixed

- We were incorrectly doing entity parsing inside CDATA sections. This caused
  `&` to be rejected as incorrect in such a section, but it should be
  acccepted. Now we accept the text in these sections as-is instead and don't
  try to do any processing on them. Thanks to dsrich for the report! Fixes
  https://github.com/faassen/xot/issues/3

## [0.13.2] - 2023-05-02

### Added

- Make Xot a bit more generous about what encodings it supports: lowercase `utf-8` is
  accepted, as well as `us-ascii`. Note that `us-ascii` at present is handled just
  like `UTF-8`, and no invalid ASCII is detected by the parser.

## [0.13.1] - 2023-03-23

### Added

- New `xot.level_order()` method to traverse the tree in level order.

## [0.13.0] - 2023-02-23

### Added

- New `xot.xml_space_name()` to access the `NameId` of the
  `xml:space` attribute.

### Changed

- `remove_insignificant_whitespace` now respects `xml:space="preserve"`.

- Pretty printing now also respects `xml:space="preserve"`.

- Expose `inherited_prefixes` method which gets the prefixes that a node
  inherits but does not define locally.

## [0.12.1] - 2023-02-15

### Changed

- Proptest won't create carriage return as text content anymore, as
  that is supposed to be normalized by XML anyway.

## [0.12.0] - 2023-02-08

### Added

- Expose `no_namespace`, `empty_prefix`, `xml_namepace` and `xml_prefix`
  methods.

### Changed

- Xot doesn't take a lifetime specifier anymore.

- The lifetime restriction on `Xot.add_name` and `Xot.add_name_ns` has been
  lifted; you can now pass in more short-lived names. Internally `Cow` was
  previously in use to store names, but that was a minor optimization far more
  trouble than it was worth.

## [0.11.8] - 2023-02-07

### Added

- Added `remove_insigificant_whitespace` to the `Xot` API. This removes
  insignificant whitespace nodes from the tree.

## [0.11.7] - 2023-02-01

### Added

- Added `is_prefix_defined` to the `Xot` API to check whether a prefix is
  defined in the context of a node.

- Added `prefix_for_namespace` to the `Xot` API to get a defined prefix for
  that namespace in the context of a node, or `None` if it isn't defined.

## [0.11.6] - 2023-02-01

### Added

- Added `unresolved_namespaces` to the `Xot` API. This gives namespaces that
  aren't defined by prefixes within the given node or its descendants.

- Changed the behavior of `clone_with_prefixes` to be more accurate: it only
  includes prefixes for unresolved namespaces, not any additional ones.

## [0.11.5] - 2023-01-26

### Added

- Added `compare` and `compare_ignore_attributes to the `Element` API. This
  allows you to compare two elements semantically, with the option to ignore
  some attributes during the comparison.

- Added `xot.compare_children`. This allows you to compare the children of
  two element nodes, even though they may have different names or attributes.

## [0.11.4] - 2023-01-17

### Fixed

- Fixed another bug with text node consolidation; `xot.clone` is now robust
  against text node consolidation.

## [0.11.3] - 2023-01-17

### Fixed

- Fixed a bug with text node consolidation. In some cases when using insert
  before or insert after, a node was not properly consolidated.

## [0.11.2] - 2023-01-16

### Added

- Make `xot::Value` hashable. This means `Element`, `Text`, `Comment`, etc are
  now hashable too.

## [0.11.1] - 2023-01-13

### Added

- Added the ability to turn off text node consolidation. Use
  `Xot::set_text_consolidation(false)` to turn it off or on again.

## [0.11.0] - 2023-01-13

### Added

- Add pretty printing support. You can use `Xot::with_serialize_options()` to
  turn on pretty printing before serialization.

- Added a `Xot::tokens` and `Xot::pretty_tokens` API to extract raw serialized
  XML for custom rendering. This replaces the `SerializerWriter` API.

### Breaking changes

- Revised the serialization API yet again as it was getting too complicated:

  - To serialize any node (including the root) to a `Xot::to_string`. This
    replaces `Xot::serialize_to_string` and `Xot::serialize_node_to_string`.

  - Serialization never takes a mutable self anymore.

  - To write to a `Write`, use `Xot::write`. This replaces `Xot::serialize` and
    `Xot::serialize_node`.

  - Previously serialization to string did not return an error. This has been
    changed so that an error is returned as well.

  - Serialization now errors if a namespace URI is encountered with a missing
    prefix. This can be fixed by calling `create_missing_prefixes` just before
    serialization, but you need to do this manually; it won't be automatic
    anymore.

- Removed the `SerializerWriter` trait introduced by `0.10.0`, and
  `serialize_with_writer`. The required functionality (and more) is now exposed
  through `Xot::tokens` and `Xot::pretty_tokens`.

## [0.10.3] - 2023-01-09

### Changed

- We still haven't gotten the serialization right. The `SerializerWriter` was
  taking fullname parameters, but that breaks expectations. Instead let the
  `SerializerWriter` own the fullname serializer so it can construct names as
  needed.

## [0.10.2] - 2023-01-09

### Changed

- `serialize_with_writer` now only takes an immutable self, and it's the only
  way to serialize with a writer. It does not attempt to add missing prefixes
  anymore.

## [0.10.1] - 2023-01-09

### Added

- `namespace_for_name` to allow access of the namespace id of a particular
  name.

## [0.10.0] - 2023-01-09

### Added

- New `serialize_with_writer` and `serialize_node_with_writer` methods. These
  can be used to customize the XML write process using the new
  `SerializerWriter` trait. The `StringWriter` struct is also provided and is
  useful to implement this trait.

### Changed

- `serialize` and `serialize_node` now do return an error object, as IO may
  go wrong during the underlying write process and we don't want to hide this.

## [0.9.2] - 2023-01-05

### Fixed

- Now actually working `xot::proptest::arb_xml_root_with_config`.

## [0.9.1] - 2023-01-05

### Fixed

- Fixed incomprehensible text in `element_unwrap` error.

### Added

- Added `xot::proptest::arb_xml_root_with_config` which allows configuration of
  what is generated. Right now you can only turn on or off generation of
  comments and processing instructions outside of the document element.

## [0.9.0] - 2023-01-05

### Added

- `proptest` support. If you enable the `proptest` feature you can use
  `xot::proptest::arb_xml_root` in your own proptests. This generates a
  arbitrary XML document out of a fixed set of elements, attributes, namespace
  prefixes and namespaces.

- Elements in the Xot tree are now more lightweight now as they don't
  carry an inverse map of namespace to prefix anymore.

### Fixed

- Rework the way serialization works for namespaces, fixing various bugs
  surrounding serialization of missing prefixes.

### Removed

- Remove the `get_prefix` method on `Element`. A namespace can have multiple
  prefixes, and the API isn't really very useful.

## [0.8.0] - 2023-01-02

## Added

- `text_content_mut` and `text_content_str` now work with elements without
  any content too.

## Fixed

- Previously we couldn't wrap standalone elements with `element_wrap`, now we
  can.

## [0.7.1] - 2022-12-22

### Fixed

- Fixed a bug in `compare`: if values were the same but the structure was
  different, compare incorrectly returned `true`.

## [0.7.0] - 2022-12-21

### Added

- Lots of examples in the API docs.

### Breaking

- `new_root` has changed and requires you to pass in an element. This is
  because otherwise you could inadvertently create XML documents that are
  not allowed - a document element is required.

## [0.6.1] - 2022-12-20

### Added

- It's now possible to call `element_wrap` on the document element as well.

## [0.6.0] - 2022-12-20

### Added

- It's not possible to apply `element_unwrap` to the document element,
  if and only if it has a single child node that's an element.

- It's allowed to use `replace` on the document element as well, if it
  doesn't break the rule and the document element remains.

- Added new `is_document_element` accessor to check whether a node
  is the document element.

## [0.5.1] - 2022-12-16

### Fixed

- Fixed a bug where we couldn't clone a document root.

## [0.5.0] - 2022-12-16

### Added

- Made built in escaping rules less aggressive for serialization; text values
  now only escape `<` and `&`, and attributes escape those and `'` and '"`.

- `xot.new_root()` which allows you to create a new document from scratch
  without parsing.

- `xot.text_content`, `xot.text_content_mut` and `xot.text_content_str` for
  convenience access to the text content of an element.

- `xot.compare` which allows you to compare two nodes for semantic equality.

### Performance

- More efficient handling of lookup of name, prefix ids and namespace ids.

## [0.4.0] - 2022-12-14

### Added

- Export `ToNamespace` and `Attribute` types.

- Add documentation for `NodeEdge` variants.

## [0.3.2] - 2022-12-13

### Fixed

- `serialize_node_to_string` generated duplicate namespace declarations.

## [0.3.1] - 2022-12-13

- Make `NameId`, `PrefixId` and `NamespaceId` have `Ord`.

## [0.3.0] - 2022-12-13

- Add new `name_ns_str`, `namespace_str` and `prefix_str` methods to access
  name information.

- Changed the processing instruction API to not use `get_target` and
  `get_value` but instead to use `target()` and `value()`.

- Make `NameId`, `NamespaceId` and `PrefixId` public.

- Documentation tweaks.

- Tweak `Cargo.toml`

## [0.2.0] - 2022-12-12

Initial public release.

<!-- next-url -->
[Unreleased]: https://github.com/faassen/xot/compare/v0.23.0...HEAD
[0.23.0]: https://github.com/faassen/xot/compare/v0.22.0...v0.23.0
[0.22.0]: https://github.com/faassen/xot/compare/v0.21.0...v0.22.0
[0.21.0]: https://github.com/faassen/xot/compare/v0.20.0...v0.21.0

[0.20.0]: https://github.com/faassen/xot/compare/v0.19.0...v0.20.0
[0.19.0]: https://github.com/faassen/xot/compare/v0.18.0...v0.19.0
[0.18.0]: https://github.com/faassen/xot/compare/v0.17.0...v0.18.0
[0.17.0]: https://github.com/faassen/xot/compare/v0.16.0...v0.17.0
[0.16.0]: https://github.com/faassen/xot/compare/v0.15.0...v0.16.0
[0.15.0]: https://github.com/faassen/xot/compare/v0.14.0...v0.15.0
[0.14.0]: https://github.com/faassen/xot/compare/v0.13.5...v0.14.0
[0.13.5]: https://github.com/faassen/xot/compare/v0.13.4...v0.13.5
[0.13.4]: https://github.com/faassen/xot/compare/v0.13.3...v0.13.4
[0.13.3]: https://github.com/faassen/xot/compare/v0.13.2...v0.13.3
[0.13.2]: https://github.com/faassen/xot/compare/v0.13.1...v0.13.2
[unreleased]: https://github.com/faassen/xot/compare/v0.13.0...v0.13.1
[unreleased]: https://github.com/faassen/xot/compare/v0.12.1...v0.13.0
[unreleased]: https://github.com/faassen/xot/compare/v0.12.0...v0.12.1
[unreleased]: https://github.com/faassen/xot/compare/v0.11.8...v0.12.0
[unreleased]: https://github.com/faassen/xot/compare/v0.11.7...v0.11.8
[unreleased]: https://github.com/faassen/xot/compare/v0.11.6...v0.11.7
[unreleased]: https://github.com/faassen/xot/compare/v0.11.5...v0.11.6
[unreleased]: https://github.com/faassen/xot/compare/v0.11.4...v0.11.5
[unreleased]: https://github.com/faassen/xot/compare/v0.11.3...v0.11.4
[unreleased]: https://github.com/faassen/xot/compare/v0.11.2...v0.11.3
[unreleased]: https://github.com/faassen/xot/compare/v0.11.1...v0.11.2
[unreleased]: https://github.com/faassen/xot/compare/v0.11.0...v0.11.1
[unreleased]: https://github.com/faassen/xot/compare/v0.10.3...v0.11.0
[unreleased]: https://github.com/faassen/xot/compare/v0.10.2...v0.10.3
[unreleased]: https://github.com/faassen/xot/compare/v0.10.1...v0.10.2
[unreleased]: https://github.com/faassen/xot/compare/v0.10.0...v0.10.1
[unreleased]: https://github.com/faassen/xot/compare/v0.9.2...v0.10.0
[unreleased]: https://github.com/faassen/xot/compare/v0.9.1...v0.9.2
[unreleased]: https://github.com/faassen/xot/compare/v0.9.0...v0.9.1
[unreleased]: https://github.com/faassen/xot/compare/v0.8.0...v0.9.0
[unreleased]: https://github.com/faassen/xot/compare/v0.7.1...v0.8.0
[unreleased]: https://github.com/faassen/xot/compare/v0.7.0...v0.7.1
[unreleased]: https://github.com/faassen/xot/compare/v0.6.1...v0.7.0
[unreleased]: https://github.com/faassen/xot/compare/v0.6.0...v0.6.1
[unreleased]: https://github.com/faassen/xot/compare/v0.5.0...v0.6.0
[unreleased]: https://github.com/faassen/xot/compare/v0.4.0...v0.5.0
