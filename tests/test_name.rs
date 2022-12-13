use xot::Xot;

#[test]
fn test_name_ns_str_no_namespace() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a/>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    let name = xot.element(doc_el).unwrap().name();
    assert_eq!(xot.name_ns_str(name), ("a", ""));
}

#[test]
fn test_name_ns_str_namespace() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a xmlns="http://example.com" />"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    let name = xot.element(doc_el).unwrap().name();
    assert_eq!(xot.name_ns_str(name), ("a", "http://example.com"));
}
