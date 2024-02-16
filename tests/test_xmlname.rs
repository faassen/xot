use xot::{
    xmlname,
    xmlname::{NameIdInfo, NameStrInfo},
    NameId, Xot,
};

#[test]
fn test_owned() {
    let name = xmlname::Owned::new(
        "local".to_string(),
        "http://example.com".to_string(),
        "prefix".to_string(),
    );

    assert_eq!(name.local_name(), "local");
    assert_eq!(name.namespace(), "http://example.com");
    assert_eq!(name.prefix().unwrap(), "prefix");
    assert_eq!(name.full_name().unwrap(), "prefix:local")
}

#[test]
fn test_ref() {
    let name = xmlname::Owned::new(
        "local".to_string(),
        "http://example.com".to_string(),
        "prefix".to_string(),
    );

    let mut xot = Xot::new();
    let namespace_id = xot.add_namespace("http://example.com");
    let name_id = xot.add_name_ns("local", namespace_id);
    let prefix_id = xot.add_prefix("prefix");

    let name_ref = name.to_ref(&mut xot);
    assert_eq!(name_ref.local_name(), "local");
    assert_eq!(name_ref.namespace(), "http://example.com");
    assert_eq!(name_ref.prefix().unwrap(), "prefix");
    assert_eq!(name_ref.full_name().unwrap(), "prefix:local");
    assert_eq!(name_ref.to_owned().unwrap(), name);
    assert_eq!(name_ref.name_id(), name_id);
    assert_eq!(name_ref.namespace_id(), namespace_id);
    assert_eq!(name_ref.prefix_id().unwrap(), prefix_id);

    let name_ref_name_id: NameId = name_ref.into();
    assert_eq!(name_ref_name_id, name_id);
}

#[test]
fn test_state() {
    let name = xmlname::Owned::new(
        "local".to_string(),
        "http://example.com".to_string(),
        "prefix".to_string(),
    );

    let mut xot = Xot::new();
    let namespace_id = xot.add_namespace("http://example.com");
    let name_id = xot.add_name_ns("local", namespace_id);
    let prefix_id = xot.add_prefix("prefix");

    let name_state = name.to_state(&mut xot).unwrap();
    assert_eq!(name_state.name_id(), name_id);
    assert_eq!(name_state.namespace_id(), namespace_id);
    assert_eq!(name_state.prefix_id().unwrap(), prefix_id);

    let name_state_name_id: NameId = name_state.into();
    assert_eq!(name_state_name_id, name_id);
}

#[test]
fn test_create_element() {
    let mut xot = Xot::new();
    let name = xmlname::Create::local_name(&mut xot, "local");

    let local = xot.new_element(name);
    assert_eq!(xot.to_string(local).unwrap(), "<local/>");
}

#[test]
fn test_create_attribute_node() {
    let mut xot = Xot::new();
    let name = xmlname::Create::local_name(&mut xot, "local");

    let doc = xmlname::Create::local_name(&mut xot, "doc");
    let doc_el = xot.new_element(doc);

    let local = xot.new_attribute_node(name, "value".to_string());
    xot.append_attribute_node(doc_el, local).unwrap();
    assert_eq!(xot.to_string(doc_el).unwrap(), r#"<doc local="value"/>"#);
}
