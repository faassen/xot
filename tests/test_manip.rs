use xot::{Document, XmlData, XmlNode};

#[test]
fn test_escape_in_text() {
    let mut data = XmlData::new();
    let doc = Document::parse(r#"<doc>Data</doc>"#, &mut data).unwrap();
    let arena = &data.arena;
    let text_id = arena[arena[doc.root_node_id()].first_child().unwrap()]
        .first_child()
        .unwrap();
    let arena = &mut data.arena;
    if let XmlNode::Text(node) = arena.get_mut(text_id).unwrap().get_mut() {
        node.set("Changed".into());
    }
    assert_eq!(
        doc.serialize_to_string(&data).unwrap(),
        r#"<doc>Changed</doc>"#
    );
}
