use xot::{Document, XmlData, XmlNode};

#[test]
fn test_escape_in_text() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<a>&lt;</a>"#, &mut data).unwrap();
    let root_id = doc.root_node_id();
    // let arena = doc.arena();
    let children = doc.children(root_id).collect::<Vec<_>>();
    assert_eq!(children.len(), 1);
    let a_id = children[0];
    assert!(matches!(doc.xml_node(a_id), xot::XmlNode::Element(_)));
    let text_id = doc.first_child(a_id).unwrap();
    assert!(matches!(doc.xml_node(text_id), xot::XmlNode::Text(_)));
    match doc.xml_node(text_id) {
        XmlNode::Text(text) => {
            assert_eq!(text, "<");
        }
        _ => unreachable!(),
    }
}
