<!-- next-header -->

## [Unreleased] - ReleaseDate

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

[unreleased]: https://github.com/faassen/xot/compare/v0.11.4...HEAD
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
