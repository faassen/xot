use indextree::Arena;
use xot::Document;

#[test]
fn roundtrip() {
    let mut arena = Arena::new();
    let xml = r#"<root><a>1</a><b>2</b></root>"#;
    let doc = Document::parse(xml, &mut arena).unwrap();
    println!("{:?}", doc.root_node_id(&arena).debug_pretty_print(&arena));
    let mut buf = Vec::new();
    doc.serialize(doc.root_node_id(&arena), &mut buf, &arena)
        .unwrap();
    let output_xml = String::from_utf8(buf).unwrap();
    assert_eq!(xml, output_xml);
}
