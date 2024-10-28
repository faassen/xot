use xot::Xot;

#[test]
fn test_id_normalized_prefix_postfix() {
    let mut xot = Xot::new();
    let id_name = xot.xml_id_name();

    let a = xot.parse(r#"<a xml:id=" FOO "/>"#).unwrap();
    let doc = xot.document_element(a).unwrap();

    let id = xot.attributes(doc).get(id_name).unwrap();

    assert_eq!(id, "FOO");
}

#[test]
fn test_id_normalized_internal() {
    let mut xot = Xot::new();
    let id_name = xot.xml_id_name();

    let a = xot.parse(r#"<a xml:id="A  B"/>"#).unwrap();
    let doc = xot.document_element(a).unwrap();

    let id = xot.attributes(doc).get(id_name).unwrap();

    assert_eq!(id, "A B");
}

#[test]
fn test_id_normalized_newline() {
    let mut xot = Xot::new();
    let id_name = xot.xml_id_name();

    let a = xot.parse("<a xml:id=\"\nFOO\"/>").unwrap();
    let doc = xot.document_element(a).unwrap();

    let id = xot.attributes(doc).get(id_name).unwrap();

    // newline is cleaned up due to normal attribute value processing,
    // then subsequently cleaned up with xml:id processing
    assert_eq!(id, "FOO");
}
