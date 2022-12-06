use xot::{Document, XmlData, XmlNode};

#[test]
fn test_escape_in_text() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<a>&lt;</a>"#, &mut data).unwrap();
    let text_id = data
        .first_child(data.first_child(doc.root_node_id()).unwrap())
        .unwrap();
    assert!(matches!(data.xml_node(text_id), XmlNode::Text(_)));
    match data.xml_node(text_id) {
        XmlNode::Text(text) => {
            assert_eq!(text.get(), "<");
        }
        _ => unreachable!(),
    }
}
