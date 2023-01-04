//! Proptest support for Xot
//!
//! Proptests allow you to test for *properties* of your code that must hold
//! for arbitrary data. Xot helps you write a proptest by letting you generate
//! an arbitrary XML document.
//!
//! This can be enabled by adding the `proptest` feature to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! xot = { version = "0.9", features = ["proptest"] }
//! ```
//!
//! See the [`proptest`](https://docs.rs/proptest/latest/proptest/)
//! documentation for more information.

use proptest::prelude::*;

use crate::fixed::{FixedContent, FixedElement, FixedRoot, FixedRootContent};

const ELEMENT_NAMES: &[&str] = &["a", "b", "c", "d", "e"];
const PI_NAMES: &[&str] = &["pi1", "pi2", "pi3", "pi4", "pi5"];
const XML_STRING: &str = "[\u{000a}\u{0009}\u{000D}][\u{0020}-\u{D7FF}][\u{E000}-\u{FFFD}]*";
const XML_STRING_WITHOUT_WHITESPACE: &str = "[\u{0020}-\u{D7FF}][\u{E000}-\u{FFFD}]*";

fn arb_fixed_content() -> impl Strategy<Value = FixedContent> {
    let pi_data = XML_STRING_WITHOUT_WHITESPACE.prop_filter("non-empty string", |s| !s.is_empty());
    let comment_text = XML_STRING.prop_filter("comment", |s| !s.contains('-'));
    let leaf = prop_oneof![
        XML_STRING.prop_map(FixedContent::Text),
        comment_text.prop_map(FixedContent::Comment),
        (prop::sample::select(PI_NAMES), prop::option::of(pi_data)).prop_map(|(target, data)| {
            FixedContent::ProcessingInstruction(target.to_string(), data)
        }),
    ];

    leaf.prop_recursive(
        8,   // levels deep
        256, // maximum size of 256 nodes
        10,  // up to 10 items per collection
        |inner| {
            (
                prop::sample::select(ELEMENT_NAMES),
                prop::collection::vec(inner, 0..10),
            )
                .prop_map(|(name, children)| {
                    FixedContent::Element(FixedElement {
                        namespace: "".to_string(),
                        name: name.to_string(),
                        attributes: vec![],
                        prefixes: vec![],
                        children,
                    })
                })
        },
    )
}

prop_compose! {
    fn arb_fixed_element()(name in prop::sample::select(ELEMENT_NAMES), children in arb_fixed_content()) -> FixedElement {
        FixedElement {
            namespace: "".to_string(),
            name: name.to_string(),
            attributes: vec![],
            prefixes: vec![],
            children: vec![children],
        }
    }
}

/// Generate a random XML document.
/// This produces a value that can be converted into a `Xot` node using its
/// `xotify` method.
///
/// Example:
///
/// ```notrust
/// use xot::proptest::arb_xml_root;
/// use xot::Xot;
///
/// proptest! {
///   #[test]
///   fn test_arb_xml_can_serialize_parse(root in arb_xml_root()) {
///     let mut xot = Xot::new();
///     let node = root.xotify(&mut xot);
///     let serialized = xot.serialize_to_string(node);
///     let parsed = xot.parse(&serialized);
///     prop_assert!(parsed.is_ok(), "Cannot parse: {} {} {:?}", serialized, parsed.err().unwrap(), serialized);
///   }
/// }
/// ```
pub fn arb_xml_root() -> impl Strategy<Value = FixedRoot> {
    let before = prop::collection::vec(
        prop_oneof![
            any::<String>().prop_map(FixedRootContent::Comment),
            (any::<String>(), any::<Option<String>>())
                .prop_map(|(target, data)| FixedRootContent::ProcessingInstruction(target, data)),
        ],
        0..10,
    );
    let after = before.clone();

    (before, arb_fixed_element(), after).prop_map(|(before, document_element, after)| FixedRoot {
        before: vec![],
        document_element,
        after: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xotdata::Xot;

    proptest! {
        #[test]
        fn test_arb_xml_can_serialize_parse(fixed_root in arb_xml_root()) {
            let mut xot = Xot::new();
            let node = fixed_root.xotify(&mut xot);
            let serialized = xot.serialize_to_string(node);
            let parsed = xot.parse(&serialized);
            prop_assert!(parsed.is_ok(), "Cannot parse: {} {} {:?}", serialized, parsed.err().unwrap(), serialized);
        }
    }
}
