use xot::Xot;

#[test]
fn test_text_content_str() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a>text</a>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    assert_eq!(xot.text_content_str(doc_el), Some("text"));
}

#[test]
fn test_text_content_str_no_text() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a/>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    assert_eq!(xot.text_content_str(doc_el), Some(""));
}

#[test]
fn test_text_content_str_mixed_content() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a>text<b/></a>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    assert_eq!(xot.text_content_str(doc_el), None);
}

#[test]
fn test_compare() {
    let mut xot = Xot::new();
    let doc1 = xot.parse(r#"<a>text</a>"#).unwrap();
    let doc2 = xot.parse(r#"<a>text</a>"#).unwrap();

    assert!(xot.deep_equal(doc1, doc2));
}

#[test]
fn test_compare_different_text() {
    let mut xot = Xot::new();
    let doc1 = xot.parse(r#"<a>text A</a>"#).unwrap();
    let doc2 = xot.parse(r#"<a>text B</a>"#).unwrap();

    assert!(!xot.deep_equal(doc1, doc2));
}

#[test]
fn test_compare_different_structure() {
    let mut xot = Xot::new();
    let doc1 = xot.parse(r#"<a></a>"#).unwrap();
    let doc2 = xot.parse(r#"<a><b/></a>"#).unwrap();

    assert!(!xot.deep_equal(doc1, doc2));
}

#[test]
fn test_compare_different_namespace() {
    let mut xot = Xot::new();
    let doc1 = xot
        .parse(r#"<a xmlns="http://example.com/a"></a>"#)
        .unwrap();
    let doc2 = xot
        .parse(r#"<a xmlns="http://example.com/b"></a>"#)
        .unwrap();

    assert!(!xot.deep_equal(doc1, doc2));
}

#[test]
fn test_compare_same_attributes() {
    let mut xot = Xot::new();
    let doc1 = xot.parse(r#"<a x="X"></a>"#).unwrap();
    let doc2 = xot.parse(r#"<a x="X"></a>"#).unwrap();

    assert!(xot.deep_equal(doc1, doc2));
}

#[test]
fn test_compare_different_attributes() {
    let mut xot = Xot::new();
    let doc1 = xot.parse(r#"<a x="X"></a>"#).unwrap();
    let doc2 = xot.parse(r#"<a x="Y"></a>"#).unwrap();

    assert!(!xot.deep_equal(doc1, doc2));
}

#[test]
fn test_compare_different_attributes_extra() {
    let mut xot = Xot::new();
    let doc1 = xot.parse(r#"<a x="X"></a>"#).unwrap();
    let doc2 = xot.parse(r#"<a x="X" y="Y"></a>"#).unwrap();

    assert!(!xot.deep_equal(doc1, doc2));
}

#[test]
fn test_compare_value_the_same_structure_different() {
    let mut xot = Xot::new();
    let doc1 = xot.parse(r#"<article><body><sec id="1"><sec id="2"><title>T1</title><p>P1</p><p>P2</p></sec></sec><sec id="3"><title>T2</title><p>P3</p></sec></body></article>"#
    ).unwrap();
    let doc2 = xot.parse(r#"<article><body><sec id="1"><sec id="2"><title>T1</title><p>P1</p><p>P2</p></sec><sec id="3"><title>T2</title><p>P3</p></sec></sec></body></article>"#).unwrap();
    assert!(!xot.deep_equal(doc1, doc2));
}

#[test]
fn test_compare_children() {
    let mut xot = Xot::new();
    let root1 = xot
        .parse(r#"<a><b z="Y">Alpha<x/>Gamma</b><b>Alpha<x/></b></a>"#)
        .unwrap();
    let root2 = xot.parse(r#"<a><b z="Z">Alpha<x/>Gamma</b></a>"#).unwrap();
    let doc1 = xot.document_element(root1).unwrap();
    let doc2 = xot.document_element(root2).unwrap();
    let doc1_b0 = xot.first_child(doc1).unwrap();
    let doc1_b1 = xot.next_sibling(doc1_b0).unwrap();
    let doc2_b0 = xot.first_child(doc2).unwrap();

    assert!(xot.deep_equal_children(doc1_b0, doc2_b0));
    assert!(!xot.deep_equal_children(doc1_b1, doc2_b0));
}

#[test]
fn test_advanced_compare_text() {
    let mut xot = Xot::new();
    let doc1 = xot.parse(r#"<a>text</a>"#).unwrap();
    let doc2 = xot.parse(r#"<a>TEXT</a>"#).unwrap();
    // case insensitive compare
    assert!(xot.advanced_deep_equal(
        doc1,
        doc2,
        |_| true,
        |a, b| a.to_lowercase() == b.to_lowercase()
    ));
}

#[test]
fn test_advanced_compare_text2() {
    let mut xot = Xot::new();
    let doc1 = xot.parse(r#"<a>text</a>"#).unwrap();
    let doc2 = xot.parse(r#"<a>different</a>"#).unwrap();

    // case insensitive compare doesn't matter, it's still different
    assert!(!xot.advanced_deep_equal(
        doc1,
        doc2,
        |_| true,
        |a, b| a.to_lowercase() == b.to_lowercase()
    ));
}

#[test]
fn test_advanced_compare_attribute_text() {
    let mut xot = Xot::new();
    let doc1 = xot.parse(r#"<a alpha="alpha">text</a>"#).unwrap();
    let doc2 = xot.parse(r#"<a alpha="ALPHA">text</a>"#).unwrap();
    // no text compares as equal, so this is not equal
    assert!(xot.advanced_deep_equal(
        doc1,
        doc2,
        |_| true,
        |a, b| a.to_lowercase() == b.to_lowercase()
    ));
}

#[test]
fn test_advanced_compare_filter() {
    let mut xot = Xot::new();
    let doc1 = xot.parse(r#"<a>text<!--comment--></a>"#).unwrap();
    let doc2 = xot.parse(r#"<a>text</a>"#).unwrap();
    // compare, disregarding comments
    assert!(xot.advanced_deep_equal(doc1, doc2, |node| !xot.is_comment(node), |a, b| a == b));
}

#[test]
fn test_first_child_with_namespace() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc xmlns:foo="FOO">bar</doc>"#).unwrap();
    let element = xot.document_element(doc).unwrap();
    let value = xot.value(xot.first_child(element).unwrap());
    if let xot::Value::Text(text) = value {
        assert_eq!(text.get(), "bar");
    } else {
        unreachable!();
    }
}

#[test]
fn test_first_child_with_attribute() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc foo="FOO">bar</doc>"#).unwrap();
    let element = xot.document_element(doc).unwrap();
    let value = xot.value(xot.first_child(element).unwrap());
    if let xot::Value::Text(text) = value {
        assert_eq!(text.get(), "bar");
    } else {
        unreachable!();
    }
}

#[test]
fn test_first_child_missing_with_attribute() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc foo="FOO"/>"#).unwrap();
    let element = xot.document_element(doc).unwrap();
    assert_eq!(xot.first_child(element), None);
}

#[test]
fn test_last_child_with_attribute() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc foo="FOO">bar</doc>"#).unwrap();
    let element = xot.document_element(doc).unwrap();
    let value = xot.value(xot.last_child(element).unwrap());
    if let xot::Value::Text(text) = value {
        assert_eq!(text.get(), "bar");
    } else {
        unreachable!();
    }
}

#[test]
fn test_last_child_missing_with_attribute() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc foo="FOO"/>"#).unwrap();
    let element = xot.document_element(doc).unwrap();
    assert_eq!(xot.last_child(element), None);
}

#[test]
fn test_previous_sibling_with_attribute() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc foo="FOO">bar</doc>"#).unwrap();
    let doc = xot.document_element(doc).unwrap();
    let bar = xot.first_child(doc).unwrap();
    assert_eq!(xot.previous_sibling(bar), None);
}

#[test]
fn test_next_sibling_with_attribute() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc foo="FOO">bar</doc>"#).unwrap();
    let doc = xot.document_element(doc).unwrap();
    let bar = xot.first_child(doc).unwrap();
    assert_eq!(xot.next_sibling(bar), None);
}

#[test]
fn test_children() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc foo="FOO">bar</doc>"#).unwrap();
    let doc = xot.document_element(doc).unwrap();
    let children = xot.children(doc).collect::<Vec<_>>();
    assert_eq!(children.len(), 1);
}
