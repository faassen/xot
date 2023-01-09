<!-- next-header -->

## [Unreleased] - ReleaseDate

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

[unreleased]: https://github.com/faassen/xot/compare/v0.10.0...HEAD
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
