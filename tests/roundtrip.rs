use xot::{Document, XmlData};

#[test]
fn roundtrip() {
    let mut data = XmlData::new();
    let xml = r#"<root><a>1</a><b>2</b></root>"#;
    let doc = Document::parse(xml, &mut data).unwrap();

    let mut buf = Vec::new();
    doc.serialize(doc.root_node_id(), &mut buf).unwrap();
    let output_xml = String::from_utf8(buf).unwrap();
    assert_eq!(xml, output_xml);
}
