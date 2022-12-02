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

#[test]
fn roundtrip_ns() {
    let mut data = XmlData::new();
    let xml =
        r#"<foo:root xmlns:foo="http://example.com"><foo:a>1</foo:a><foo:b>2</foo:b></foo:root>"#;
    let doc = Document::parse(xml, &mut data).unwrap();

    let mut buf = Vec::new();
    doc.serialize(doc.root_node_id(), &mut buf).unwrap();
    let output_xml = String::from_utf8(buf).unwrap();
    assert_eq!(xml, output_xml);
}

#[test]
fn roundtrip_some_ns() {
    let mut data = XmlData::new();
    let xml = r#"<root xmlns:foo="http://example.com"><a>1</a><foo:b>2</foo:b></root>"#;
    let doc = Document::parse(xml, &mut data).unwrap();

    let mut buf = Vec::new();
    doc.serialize(doc.root_node_id(), &mut buf).unwrap();
    let output_xml = String::from_utf8(buf).unwrap();
    assert_eq!(xml, output_xml);
}

#[test]
fn roundtrip_default_ns() {
    let mut data = XmlData::new();
    let xml = r#"<root xmlns="http://example.com"><a>1</a><b>2</b></root>"#;
    let doc = Document::parse(xml, &mut data).unwrap();

    let mut buf = Vec::new();
    doc.serialize(doc.root_node_id(), &mut buf).unwrap();
    let output_xml = String::from_utf8(buf).unwrap();
    assert_eq!(xml, output_xml);
}
