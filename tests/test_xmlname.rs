use xot::{xmlname, xmlname::NameStrInfo, NameId, Xot};

#[test]
fn test_owned() {
    let name = xmlname::OwnedName::new(
        "local".to_string(),
        "http://example.com".to_string(),
        "prefix".to_string(),
    );

    assert_eq!(name.local_name(), "local");
    assert_eq!(name.namespace(), "http://example.com");
    assert_eq!(name.prefix(), "prefix");
    assert_eq!(name.full_name(), "prefix:local")
}

#[test]
fn test_ref() {
    let name = xmlname::OwnedName::new(
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
    assert_eq!(name_ref.prefix(), "prefix");
    assert_eq!(name_ref.full_name(), "prefix:local");
    assert_eq!(name_ref.to_owned(), name);
    assert_eq!(name_ref.name_id(), name_id);
    assert_eq!(name_ref.namespace_id(), namespace_id);
    assert_eq!(name_ref.prefix_id(), prefix_id);

    let name_ref_name_id: NameId = name_ref.into();
    assert_eq!(name_ref_name_id, name_id);
}

#[test]
fn test_create_element() {
    let mut xot = Xot::new();
    let name = xmlname::CreateName::name(&mut xot, "local");

    let local = xot.new_element(name);
    assert_eq!(xot.to_string(local).unwrap(), "<local/>");
}

#[test]
fn test_create_element_namespace() {
    let mut xot = Xot::new();
    let namespace = xmlname::CreateNamespace::new(&mut xot, "ex", "http://example.com");
    let name = xmlname::CreateName::namespaced(&mut xot, "local", &namespace);

    let local = xot.new_element(name);
    xot.append_namespace(local, &namespace).unwrap();
    assert_eq!(
        xot.to_string(local).unwrap(),
        r#"<ex:local xmlns:ex="http://example.com"/>"#
    );
}

#[test]
fn test_create_attribute_node() {
    let mut xot = Xot::new();
    let name = xmlname::CreateName::name(&mut xot, "local");

    let doc = xmlname::CreateName::name(&mut xot, "doc");
    let doc_el = xot.new_element(doc);

    let local = xot.new_attribute_node(name, "value".to_string());
    xot.append_attribute_node(doc_el, local).unwrap();
    assert_eq!(xot.to_string(doc_el).unwrap(), r#"<doc local="value"/>"#);
}

#[test]
fn test_create_attribute() {
    let mut xot = Xot::new();
    let el_name = xmlname::CreateName::name(&mut xot, "local");
    let attr_name = xmlname::CreateName::name(&mut xot, "attr");
    let doc_el = xot.new_element(el_name);

    xot.attributes_mut(doc_el)
        .insert(attr_name, "value".to_string());
    assert_eq!(xot.to_string(doc_el).unwrap(), r#"<local attr="value"/>"#);
}
