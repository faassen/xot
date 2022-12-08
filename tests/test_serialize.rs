use xot::XmlData;

#[test]
fn test_serialize_fragment() {
    let mut data = XmlData::new();
    let doc = data
        .parse(r#"<doc xmlns:foo="http://example.com"><foo:a/></doc>"#)
        .unwrap();
    let node = data.first_child(data.root_element(doc).unwrap()).unwrap();
    assert_eq!(
        data.serialize_fragment_to_string(node),
        r#"<foo:a xmlns:foo="http://example.com"/>"#
    );
}
