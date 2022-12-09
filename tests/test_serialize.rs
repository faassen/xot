use xot::Xot;

#[test]
fn test_serialize_node() {
    let mut xot = Xot::new();
    let doc = xot
        .parse(r#"<doc xmlns:foo="http://example.com"><foo:a/></doc>"#)
        .unwrap();
    let node = xot.first_child(xot.document_element(doc).unwrap()).unwrap();
    assert_eq!(
        xot.serialize_node_to_string(node),
        r#"<foo:a xmlns:foo="http://example.com"/>"#
    );
}
