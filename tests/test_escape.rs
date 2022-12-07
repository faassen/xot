use xot::{Document, Value, XmlData};

#[test]
fn test_escape_in_text() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<a>&lt;</a>"#, &mut data).unwrap();
    let text_id = data.first_child(data.root_element(&doc)).unwrap();
    assert!(matches!(data.value(text_id), Value::Text(_)));
    match data.value(text_id) {
        Value::Text(text) => {
            assert_eq!(text.get(), "<");
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_add_attribute_entities() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<doc/>"#, &mut data).unwrap();
    let el_id = data.root_element(&doc);
    assert!(data.name("a").is_none());
    let a = data.name_mut("a");

    if let Value::Element(element) = data.value_mut(el_id) {
        element.set_attribute(a, "Created & set".to_string());
    }
    assert_eq!(
        doc.serialize_to_string(&data).unwrap(),
        r#"<doc a="Created &amp; set"/>"#
    );
}
