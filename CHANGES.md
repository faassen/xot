## unreleased

### Added

- Export `ToNamespace` and `Attribute` types.

- Add documentation for `NodeEdge` variants.

## 0.3.2 (2022-12-13)

### Fixed

- `serialize_node_to_string` generated duplicate namespace declarations.

## 0.3.1 (2022-12-13)

- Make `NameId`, `PrefixId` and `NamespaceId` have `Ord`.

## 0.3.0 (2022-12-13)

- Add new `name_ns_str`, `namespace_str` and `prefix_str` methods to access
  name information.

- Changed the processing instruction API to not use `get_target` and
  `get_value` but instead to use `target()` and `value()`.

- Make `NameId`, `NamespaceId` and `PrefixId` public.

- Documentation tweaks.

- Tweak `Cargo.toml`

## 0.2.0 (2022-12-12)

Initial public release.
