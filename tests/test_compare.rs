use xot::Xot;

#[test]
fn test_compare_elements_same() {
    let mut xot = Xot::new();

    let a = xot.parse(r#"<a foo="FOO"/>"#).unwrap();
    let b = xot.parse(r#"<a foo="FOO"/>"#).unwrap();

    assert!(xot.deep_equal(a, b));
}

#[test]
fn test_compare_elements_different_value() {
    let mut xot = Xot::new();

    let a = xot.parse(r#"<a foo="FOO"/>"#).unwrap();
    let b = xot.parse(r#"<a foo="BAR"/>"#).unwrap();

    assert!(!xot.deep_equal(a, b));
}

#[test]
fn test_compare_elements_attribute_order_unimportant() {
    let mut xot = Xot::new();

    let a = xot.parse(r#"<a foo="FOO" bar="BAR"/>"#).unwrap();
    let b = xot.parse(r#"<a bar="BAR" foo="FOO"/>"#).unwrap();

    assert!(xot.deep_equal(a, b));
}

#[test]
fn test_compare_elements_compare_overlap() {
    let mut xot = Xot::new();

    let a = xot.parse(r#"<a foo="FOO" />"#).unwrap();
    let b = xot.parse(r#"<a foo="FOO" bar="BAR"/>"#).unwrap();

    assert!(!xot.deep_equal(a, b));
}

#[test]
fn test_compare_elements_ignore_attributes_different_value() {
    let mut xot = Xot::new();
    let bar = xot.add_name("bar");

    let a = xot.parse(r#"<a foo="FOO" bar="BAR"/>"#).unwrap();
    let b = xot.parse(r#"<a foo="FOO" bar="QUX"/>"#).unwrap();

    let a = xot.document_element(a).unwrap();
    let b = xot.document_element(b).unwrap();

    assert!(xot.shallow_equal_ignore_attributes(a, b, &[bar]));
    assert!(!xot.shallow_equal_ignore_attributes(b, a, &[]));
}

#[test]
fn test_compare_elements_ignore_attributes_ignorable_in_a() {
    let mut xot = Xot::new();
    let bar = xot.add_name("bar");

    let a = xot.parse(r#"<a foo="FOO" bar="BAR"/>"#).unwrap();
    let b = xot.parse(r#"<a foo="FOO"/>"#).unwrap();

    let a = xot.document_element(a).unwrap();
    let b = xot.document_element(b).unwrap();

    assert!(xot.shallow_equal_ignore_attributes(a, b, &[bar]));
    assert!(!xot.shallow_equal_ignore_attributes(b, a, &[]));
}

#[test]
fn test_compare_elements_ignore_attributes_ignorable_in_b() {
    let mut xot = Xot::new();
    let bar = xot.add_name("bar");

    let a = xot.parse(r#"<a foo="FOO"/>"#).unwrap();
    let b = xot.parse(r#"<a foo="FOO" bar="BAR"/>"#).unwrap();

    let a = xot.document_element(a).unwrap();
    let b = xot.document_element(b).unwrap();

    assert!(xot.shallow_equal_ignore_attributes(a, b, &[bar]));
    assert!(!xot.shallow_equal_ignore_attributes(b, a, &[]));
}
