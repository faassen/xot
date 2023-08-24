use encoding_rs::Encoding;
use xhtmlchardet::detect;

pub(crate) fn encoding(data: &[u8], hint: Option<String>) -> Option<&'static Encoding> {
    let mut cursor = std::io::Cursor::new(data);
    let charsets = detect(&mut cursor, hint).ok()?;
    // no encoding detected
    let label = if charsets.is_empty() {
        "UTF-8"
    } else {
        &charsets[0]
    };
    Encoding::for_label(label.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf8() {
        let data = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><a/>";
        let enc = encoding(data, None).unwrap();
        assert_eq!(enc.name(), "UTF-8");
    }

    #[test]
    fn test_utf8_without_declaration() {
        let data = b"<a/>";
        let enc = encoding(data, None).unwrap();
        assert_eq!(enc.name(), "UTF-8");
    }

    #[test]
    fn test_us_ascii() {
        let data = b"<?xml version=\"1.0\" encoding=\"us-ascii\"?><a/>";
        let enc = encoding(data, None).unwrap();
        // this is a superset, so should be okay?
        assert_eq!(enc.name(), "windows-1252");
    }

    #[test]
    fn test_iso8859_1() {
        let data = b"<?xml version=\"1.0\" encoding=\"iso-8859-1\"?><a/>";
        let enc = encoding(data, None).unwrap();
        // windows-1252 is a superset of 8859-1
        assert_eq!(enc.name(), "windows-1252");
    }
}
