use xot::{Error, ParseError, Xot};

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

#[test]
fn test_id_no_duplicates_in_doc() {
    let mut xot = Xot::new();
    let err = xot
        .parse(r#"<doc><a xml:id="FOO"/><b xml:id="FOO"/></doc>"#)
        //        012345678901234567890123456789012345678901234567
        .unwrap_err();
    match err {
        ParseError::DuplicateId(value, span) => {
            assert_eq!(value, "FOO");
            assert_eq!(span, (33..36).into());
        }
        _ => panic!("unexpected error"),
    }
}

#[test]
fn test_id_that_are_not_duplicates() {
    let mut xot = Xot::new();
    let r = xot.parse(r#"<doc><a xml:id="FOO"/><b xml:id="BAR"/></doc>"#);
    assert!(r.is_ok());
}

#[test]
fn test_other_duplicate_attributes_are_fine() {
    let mut xot = Xot::new();
    let r = xot.parse(r#"<doc><a x="FOO"/><b x="FOO"/></doc>"#);
    assert!(r.is_ok());
}

#[test]
fn xml_id_node() {
    let mut xot = Xot::new();
    let root = xot
        .parse(r#"<doc><a xml:id="FOO"/><b xml:id="BAR"/></doc>"#)
        .unwrap();
    let doc = xot.document_element(root).unwrap();
    let a = xot.first_child(doc).unwrap();
    let b = xot.next_sibling(a).unwrap();

    assert_eq!(xot.xml_id_node(root, "FOO"), Some(a));
    assert_eq!(xot.xml_id_node(root, "BAR"), Some(b));
    assert_eq!(xot.xml_id_node(root, "QUX"), None);
}
