use xot::{ParseError, Span, SpanInfoKey, Xot};

const US_ASCII: &str = include_str!("fixtures/us-ascii.xml");

#[test]
fn test_unclosed_tag() {
    let mut xot = Xot::new();
    let err = xot.parse(r#"<a><b></a>"#).unwrap_err();
    assert!(matches!(err, ParseError::InvalidCloseTag(_, _, _)));
}

#[test]
fn test_unclosed_tag_at_end() {
    let mut xot = Xot::new();
    let err = xot.parse(r#"<a>"#).unwrap_err();
    assert!(matches!(err, ParseError::UnclosedTag(_)));
}

#[test]
fn test_duplicate_attributes() {
    let mut xot = Xot::new();
    let err = xot.parse(r#"<a x="x" x="y"/>"#).unwrap_err();
    let s = match err {
        ParseError::DuplicateAttribute(s, _) => s,
        _ => unreachable!(),
    };
    assert_eq!(s, "x");
}

#[test]
fn test_duplicate_attributes_ns() {
    let mut xot = Xot::new();
    let err = xot
        .parse(r#"<a xmlns:foo="http://example.com" foo:x="x" foo:x="y"/>"#)
        .unwrap_err();
    let s = match err {
        ParseError::DuplicateAttribute(s, _) => s,
        _ => unreachable!(),
    };
    assert_eq!(s, "foo:x");
}

#[test]
fn test_parse_xml_declaration() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<?xml version="1.0" encoding="UTF-8"?><a/>"#);
    assert!(doc.is_ok());
}

#[test]
fn test_parse_xml_ignore_standalone_declaration() {
    let mut xot = Xot::new().ignore_standalone_declaration(true);
    let doc = xot.parse(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><a/>"#);
    assert!(doc.is_ok());

    let mut xot = Xot::new().ignore_standalone_declaration(true);
    let doc = xot.parse(r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?><a/>"#);
    assert!(doc.is_ok());
}

#[test]
fn test_encoding_lowercase_utf8() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<?xml version="1.0" encoding="utf-8"?><a/>"#);
    assert!(doc.is_ok());
}

#[test]
fn test_encoding_us_ascii() {
    let mut xot = Xot::new();
    let doc = xot.parse(US_ASCII);
    assert!(doc.is_ok());
}

#[test]
fn test_unknown_prefix() {
    let mut xot = Xot::new();
    let err = xot.parse(r#"<a><foo:b></a>"#).unwrap_err();
    if let ParseError::UnknownPrefix(s, _) = err {
        assert_eq!(s, "foo");
    } else {
        unreachable!();
    }
}

#[test]
fn test_parse_non_static() -> Result<(), ParseError> {
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

#[test]
fn test_ampersand() -> Result<(), ParseError> {
    let mut xot = Xot::new();
    let err = xot.parse(r#"<a>&</a>"#).unwrap_err();
    assert!(matches!(err, ParseError::UnclosedEntity(_, _)));
    Ok(())
}

#[test]
fn test_ampersand_in_cdata() -> Result<(), ParseError> {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a><![CDATA[&]]></a>"#)?;
    let doc_el = xot.document_element(doc).unwrap();
    let txt = xot.text_content_str(doc_el).unwrap();
    assert_eq!(txt, "&");
    Ok(())
}

#[test]
fn test_parse_with_span_info_element_start_unprefixed() {
    let mut xot = Xot::new();
    let (doc, span_info) = xot.parse_with_span_info(r#"<a></a>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    assert_eq!(
        span_info.get(SpanInfoKey::ElementStart(doc_el)).unwrap(),
        &Span::new(1, 2)
    );
}

#[test]
fn test_parse_with_span_info_element_start_prefixed() {
    let mut xot = Xot::new();
    let (doc, span_info) = xot
        .parse_with_span_info(r#"<foo:a xmlns:foo="http://example.com/foo"></foo:a>"#)
        .unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    assert_eq!(
        span_info.get(SpanInfoKey::ElementStart(doc_el)).unwrap(),
        &Span::new(1, 6)
    );
}

#[test]
fn test_parse_with_span_info_element_start_unprefixed_nested() {
    let mut xot = Xot::new();
    let (doc, span_info) = xot.parse_with_span_info(r#"<a><b></b></a>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    let b_el = xot.first_child(doc_el).unwrap();

    assert_eq!(
        span_info.get(SpanInfoKey::ElementStart(b_el)).unwrap(),
        &Span::new(4, 5)
    );
}

#[test]
fn test_parse_with_span_info_attribute_name_unprefixed() {
    let mut xot = Xot::new();
    let (doc, span_info) = xot.parse_with_span_info(r#"<a b="B"></a>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    let attribute_name = xot.name("b").unwrap();

    assert_eq!(
        span_info
            .get(SpanInfoKey::AttributeName(doc_el, attribute_name))
            .unwrap(),
        &Span::new(3, 4)
    );
}

#[test]
fn test_parse_with_span_info_attribute_value_unprefixed() {
    let mut xot = Xot::new();
    let (doc, span_info) = xot.parse_with_span_info(r#"<a b="B"></a>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    let attribute_name = xot.name("b").unwrap();

    assert_eq!(
        span_info
            .get(SpanInfoKey::AttributeValue(doc_el, attribute_name))
            .unwrap(),
        &Span::new(6, 7)
    );
}

#[test]
fn test_parse_with_span_info_attribute_name_prefixed() {
    let mut xot = Xot::new();
    let (doc, span_info) = xot
        .parse_with_span_info(r#"<a foo:b="B" xmlns:foo="http://example.com/foo"></a>"#)
        .unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    let ns = xot.namespace("http://example.com/foo").unwrap();
    let attribute_name = xot.name_ns("b", ns).unwrap();

    assert_eq!(
        span_info
            .get(SpanInfoKey::AttributeName(doc_el, attribute_name))
            .unwrap(),
        &Span::new(3, 8)
    );
}

#[test]
fn test_parse_with_span_info_attribute_value_prefixed() {
    let mut xot = Xot::new();
    let (doc, span_info) = xot
        .parse_with_span_info(r#"<a foo:b="B" xmlns:foo="http://example.com/foo"></a>"#)
        .unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    let ns = xot.namespace("http://example.com/foo").unwrap();
    let attribute_name = xot.name_ns("b", ns).unwrap();

    assert_eq!(
        span_info
            .get(SpanInfoKey::AttributeValue(doc_el, attribute_name))
            .unwrap(),
        &Span::new(10, 11)
    );
}

#[test]
fn test_parse_with_span_info_end_normal() {
    let mut xot = Xot::new();
    let (doc, span_info) = xot.parse_with_span_info(r#"<a></a>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    assert_eq!(
        span_info.get(SpanInfoKey::ElementEnd(doc_el)).unwrap(),
        &Span::new(3, 7)
    );
}

#[test]
fn test_parse_with_span_info_empty() {
    let mut xot = Xot::new();
    let (doc, span_info) = xot.parse_with_span_info(r#"<a/>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    assert_eq!(
        span_info.get(SpanInfoKey::ElementEnd(doc_el)).unwrap(),
        &Span::new(2, 4)
    );
}

#[test]
fn test_parse_with_span_info_text() {
    let mut xot = Xot::new();
    let (doc, span_info) = xot.parse_with_span_info(r#"<a>text</a>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    let text = xot.first_child(doc_el).unwrap();
    assert_eq!(
        span_info.get(SpanInfoKey::Text(text)).unwrap(),
        &Span::new(3, 7)
    );
}

#[test]
fn test_parse_with_span_info_comment() {
    let mut xot = Xot::new();
    let (doc, span_info) = xot
        .parse_with_span_info(r#"<a><!--comment--></a>"#)
        .unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    let comment = xot.first_child(doc_el).unwrap();
    assert_eq!(
        span_info.get(SpanInfoKey::Comment(comment)).unwrap(),
        &Span::new(7, 14)
    );
}

#[test]
fn test_parse_with_span_info_pi_target() {
    let mut xot = Xot::new();
    let (doc, span_info) = xot.parse_with_span_info(r#"<a><?pi?></a>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    let pi = xot.first_child(doc_el).unwrap();
    assert_eq!(
        span_info.get(SpanInfoKey::PiTarget(pi)).unwrap(),
        &Span::new(5, 7)
    );
}

#[test]
fn test_parse_with_span_info_pi_content() {
    let mut xot = Xot::new();
    let (doc, span_info) = xot
        .parse_with_span_info(r#"<a><?pi content?></a>"#)
        .unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    let pi = xot.first_child(doc_el).unwrap();
    assert_eq!(
        span_info.get(SpanInfoKey::PiContent(pi)).unwrap(),
        &Span::new(8, 15)
    );
}

#[test]
fn test_parse_consolidated_cdata_text() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a><![CDATA[foo]]>bar</a>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    assert_eq!(xot.children(doc_el).count(), 1);
    let txt = xot.text_content_str(doc_el).unwrap();
    assert_eq!(txt, "foobar");
}

#[test]
fn test_parse_consolidated_text_cdata() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a>foo<![CDATA[bar]]></a>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    assert_eq!(xot.children(doc_el).count(), 1);
    let txt = xot.text_content_str(doc_el).unwrap();
    assert_eq!(txt, "foobar");
}

#[test]
fn test_parse_consolidated_text_cdata_text() {
    let mut xot = Xot::new();
    let doc = xot.parse(r#"<a>foo<![CDATA[bar]]>baz</a>"#).unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    assert_eq!(xot.children(doc_el).count(), 1);
    let txt = xot.text_content_str(doc_el).unwrap();
    assert_eq!(txt, "foobarbaz");
}

#[test]
fn test_span_for_cdata() {
    let mut xot = Xot::new();
    let (doc, span_info) = xot
        .parse_with_span_info(r#"<a><![CDATA[foo]]></a>"#)
        .unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    let txt = xot.first_child(doc_el).unwrap();
    assert_eq!(
        span_info.get(SpanInfoKey::Text(txt)).unwrap(),
        &Span::new(12, 15)
    );
}

#[test]
fn test_span_for_cdata_text() {
    let mut xot = Xot::new();
    let (doc, span_info) = xot
        .parse_with_span_info(r#"<a><![CDATA[foo]]>bar</a>"#)
        .unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    let txt = xot.first_child(doc_el).unwrap();
    assert_eq!(
        span_info.get(SpanInfoKey::Text(txt)).unwrap(),
        &Span::new(12, 21)
    );
}

#[test]
fn test_span_for_text_cdata() {
    let mut xot = Xot::new();
    let (doc, span_info) = xot
        .parse_with_span_info(r#"<a>foo<![CDATA[bar]]></a>"#)
        .unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    let txt = xot.first_child(doc_el).unwrap();
    assert_eq!(
        span_info.get(SpanInfoKey::Text(txt)).unwrap(),
        &Span::new(3, 18)
    );
}

#[test]
fn test_span_for_cdata_cdata() {
    let mut xot = Xot::new();
    let (doc, span_info) = xot
        .parse_with_span_info(r#"<a><![CDATA[foo]]><![CDATA[bar]]></a>"#)
        .unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    let txt = xot.first_child(doc_el).unwrap();
    assert_eq!(
        span_info.get(SpanInfoKey::Text(txt)).unwrap(),
        &Span::new(12, 30)
    );
}

#[test]
fn test_parse_should_reject_multiple_elements_in_document() {
    let mut xot = Xot::new();
    let err = xot.parse(r#"<a/><b/>"#).unwrap_err();
    assert!(matches!(
        err,
        ParseError::XmlParser(xmlparser::Error::UnknownToken(_), _)
    ));
}

#[test]
fn test_parse_should_reject_no_elements_in_document() {
    let mut xot = Xot::new();
    let err = xot.parse(r#""#).unwrap_err();
    assert!(matches!(err, ParseError::NoElementAtTopLevel(_)))
}

#[test]
fn test_parse_should_reject_text_in_document() {
    let mut xot = Xot::new();
    let err = xot.parse(r#"text"#).unwrap_err();
    assert!(matches!(
        err,
        ParseError::XmlParser(xmlparser::Error::UnknownToken(_), _)
    ));
}

#[test]
fn test_parse_should_reject_no_elements_in_document_with_comment() {
    let mut xot = Xot::new();
    let err = xot.parse(r#"<!-- comment -->"#).unwrap_err();
    assert!(matches!(err, ParseError::NoElementAtTopLevel(_)))
}

#[test]
fn test_parse_fragment_multiple_elements() {
    let mut xot = Xot::new();
    let doc = xot.parse_fragment(r#"<a/><b/>"#).unwrap();
    assert_eq!(xot.children(doc).count(), 2);
}

#[test]
fn test_parse_fragment_no_elements() {
    let mut xot = Xot::new();
    let doc = xot.parse_fragment(r#""#).unwrap();
    assert_eq!(xot.children(doc).count(), 0);
}

#[test]
fn test_parse_fragment_no_elements_just_comment() {
    let mut xot = Xot::new();
    let doc = xot.parse_fragment(r#"<!-- comment -->"#).unwrap();
    assert_eq!(xot.children(doc).count(), 1);
}

#[test]
fn test_parse_fragment_no_elements_just_text() {
    let mut xot = Xot::new();
    let doc = xot.parse_fragment(r#"text"#).unwrap();
    assert_eq!(xot.children(doc).count(), 1);
}

#[test]
fn test_parse_fragment_element_with_text_after() {
    let mut xot = Xot::new();
    let doc = xot.parse_fragment(r#"<a/>text"#).unwrap();
    assert_eq!(xot.children(doc).count(), 2);
}

#[test]
fn test_parse_fragment_error_span() {
    let mut xot = Xot::new();
    let err = xot.parse_fragment(r#"<a><b></a>"#).unwrap_err();
    assert!(matches!(err, ParseError::InvalidCloseTag(_, _, _)));
    let span = err.span();
    assert_eq!(span, (8..9).into());
}

#[test]
fn test_parse_fragment_with_span_info() {
    let mut xot = Xot::new();
    let (doc, span_info) = xot
        .parse_fragment_with_span_info(r#"<a><b></b></a>"#)
        .unwrap();
    let doc_el = xot.document_element(doc).unwrap();
    let b_el = xot.first_child(doc_el).unwrap();
    assert_eq!(
        span_info.get(SpanInfoKey::ElementStart(b_el)).unwrap(),
        &Span::new(4, 5)
    );
}
