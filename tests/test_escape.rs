use xot::{Value, Xot};

#[test]
fn test_escape_in_text() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a>&lt;</a>"#).unwrap();
    let text_id = xot.first_child(xot.document_element(doc).unwrap()).unwrap();
    assert!(matches!(xot.value(text_id), Value::Text(_)));
    match xot.value(text_id) {
        Value::Text(text) => {
            assert_eq!(text.get(), "<");
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_add_attribute_entities() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<doc/>"#).unwrap();
    let el_id = xot.document_element(doc).unwrap();
    assert!(xot.name("a").is_none());
    let a = xot.add_name("a");

    if let Value::Element(element) = xot.value_mut(el_id) {
        element.set_attribute(a, "Created & set".to_string());
    }
    assert_eq!(
        xot.to_string(doc).unwrap(),
        r#"<doc a="Created &amp; set"/>"#
    );
}
