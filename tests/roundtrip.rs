use rstest::rstest;

use xot::Xot;

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
  ),
  (
      "attribute",
      r#"<root foo="bar"/>"#,
  ),
  ( 
      "attribute in namespace",
      r#"<root xmlns:foo="http://example.com" foo:bar="baz"/>"#,
  ),
  (
      "escape character in text",
      r#"<root>&lt;a/&gt;</root>"#,
  ),
  (
    "comment",
    r#"<root><!-- comment --></root>"#,
  ),
  (
    "processing instruction without data",
    r#"<root><?pi foo?></root>"#,
  ),
  (
    "processing instruction with data",
    r#"<root><?pi foo bar?></root>"#,
  )
)] value: RoundTripEntry) {
    let (name, xml) = value;
    let mut xot = Xot::new();
    let doc = xot.parse(xml).unwrap();
    let output_xml = xot.serialize_to_string(doc);
    assert_eq!(xml, &output_xml, "roundtrip failed for: {}", name);
}
