use xot::{Document, XmlData, XmlNode};

#[test]
fn test_escape_in_text() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<a>&lt;</a>"#, &mut data).unwrap();
    // let arena = doc.arena();
    let text_id = data.arena[data.arena[doc.root_node_id()].first_child().unwrap()]
        .first_child()
        .unwrap();
    assert!(matches!(data.arena[text_id].get(), XmlNode::Text(_)));
    match data.arena[text_id].get() {
        XmlNode::Text(text) => {
            assert_eq!(text.get(), "<");
        }
        _ => unreachable!(),
    }
}
