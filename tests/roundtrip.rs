use rstest::rstest;

use xot::{Document, XmlData};

type RoundTripEntry = (&'static str, &'static str);

#[rstest]
fn roundtrip(#[values(    
    ("basic", r#"<root><a>1</a><b>2</b></root>"#),
    ("self closing", r#"<root/>"#),
    (
      "namespace prefix",
      r#"<foo:root xmlns:foo="http://example.com"><foo:a>1</foo:a><foo:b>2</foo:b></foo:root>"#,
    ),
    (
      "some prefixed",
      r#"<root xmlns:foo="http://example.com"><a>1</a><foo:b>2</foo:b></root>"#,
  ),
  (
      "default namespace",
      r#"<root xmlns="http://example.com"><a>1</a><b>2</b></root>"#,
  ),
  (
      "prefix shadowing",
      r#"<foo:root xmlns:foo="http://outer.com"><foo:a xmlns:foo="http://inner.com"><foo:inner/></foo:a></foo:root>"#,
  ))] value: RoundTripEntry) {
    let (name, xml) = value;
    let mut data = XmlData::new();
    let doc = Document::parse(xml, &mut data).unwrap();
    let mut buf = Vec::new();
    doc.serialize(doc.root_node_id(), &mut buf).unwrap();
    let output_xml = String::from_utf8(buf).unwrap();
    assert_eq!(xml, &output_xml, "roundtrip failed for: {}", name);
}
