use xot::{Error, Xot};

#[test]
fn test_unclosed_tag() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a><b></a>"#);
    assert!(matches!(doc, Err(Error::InvalidCloseTag(_, _))));
}

#[test]
fn test_unclosed_tag_at_end() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a>"#);
    assert!(matches!(doc, Err(Error::UnclosedTag)));
}

#[test]
fn test_duplicate_attributes() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a x="x" x="y"/>"#);
    if let Err(Error::DuplicateAttribute(s)) = doc {
        assert_eq!(s, "x");
    } else {
        unreachable!();
    }
}

#[test]
fn test_duplicate_attributes_ns() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a xmlns:foo="http://example.com" foo:x="x" foo:x="y"/>"#);
    if let Err(Error::DuplicateAttribute(s)) = doc {
        assert_eq!(s, "foo:x");
    } else {
        unreachable!();
    }
}

#[test]
fn test_parse_xml_declaration() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<?xml version="1.0" encoding="UTF-8"?><a/>"#);
    assert!(doc.is_ok());
}

#[test]
fn test_encoding_lowercase_utf8() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<?xml version="1.0" encoding="utf-8"?><a/>"#);
    assert!(doc.is_ok());
}

#[test]
fn test_unknown_prefix() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a><foo:b></a>"#);
    if let Err(Error::UnknownPrefix(s)) = doc {
        assert_eq!(s, "foo");
    } else {
        unreachable!();
    }
}

#[test]
fn test_parse_non_static() -> Result<(), Error> {
    let mut xot = Xot::new();
    let mut xml = String::new();
    xml.push('<');
    xml.push('a');
    xml.push('>');
    xml.push('<');
    xml.push('/');
    xml.push('a');
    xml.push('>');
    let doc = xot.parse(&xml)?;
    drop(xml);
    let doc_el = xot.document_element(doc).unwrap();
    let el = xot.element(doc_el).unwrap();
    assert_eq!(xot.name_ns_str(el.name()), ("a", ""));
    Ok(())
}
