#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct XmlNameOwned {
    local_name: String,
    // the empty namespace means no namespace
    uri: String,
    // the empty prefix means no prefix
    prefix: String,
}
