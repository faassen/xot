use std::borrow::Cow;

use crate::error::Error;

pub(crate) fn parse_predefined_entities(content: Cow<str>) -> Result<Cow<str>, Error> {
    let mut result = String::new();
    let mut chars = content.chars();
    let mut entity_seen = false;
    while let Some(c) = chars.next() {
        if c == '&' {
            let mut entity = String::new();
            let mut is_complete = false;
            for c in chars.by_ref() {
                if c == ';' {
                    is_complete = true;
                    break;
                }
                entity.push(c);
            }
            if !is_complete {
                return Err(Error::UnclosedEntity(entity));
            }
            entity_seen = true;
            match entity.as_str() {
                "amp" => result.push('&'),
                "apos" => result.push('\''),
                "gt" => result.push('>'),
                "lt" => result.push('<'),
                "quot" => result.push('"'),
                _ => return Err(Error::InvalidEntity(entity)),
            }
        } else {
            result.push(c);
        }
    }

    if !entity_seen {
        Ok(content)
    } else {
        Ok(result.into())
    }
}

pub(crate) fn serialize_predefined_entities(content: Cow<str>) -> Cow<str> {
    let mut result = String::new();
    let mut entity_seen = false;
    for c in content.chars() {
        match c {
            '&' => {
                entity_seen = true;
                result.push_str("&amp;")
            }
            '\'' => {
                entity_seen = true;
                result.push_str("&apos;")
            }
            '>' => {
                entity_seen = true;
                result.push_str("&gt;")
            }
            '<' => {
                entity_seen = true;
                result.push_str("&lt;")
            }
            '"' => {
                entity_seen = true;
                result.push_str("&quot;")
            }
            _ => result.push(c),
        }
    }

    if !entity_seen {
        content
    } else {
        result.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let text = "A &amp; B";
        assert_eq!(parse_predefined_entities(text.into()).unwrap(), "A & B");
    }

    #[test]
    fn test_parse_multiple() {
        let text = "&amp;&apos;&gt;&lt;&quot;";
        assert_eq!(parse_predefined_entities(text.into()).unwrap(), "&'><\"");
    }

    #[test]
    fn test_parse_unknown_entity() {
        let text = "&unknown;";
        let err = parse_predefined_entities(text.into());
        if let Err(Error::InvalidEntity(entity)) = err {
            assert_eq!(entity, "unknown");
        } else {
            unreachable!();
        }
    }

    #[test]
    fn test_parse_unfinished_entity() {
        let text = "&amp";
        let err = parse_predefined_entities(text.into());
        if let Err(Error::UnclosedEntity(entity)) = err {
            assert_eq!(entity, "amp");
        } else {
            unreachable!();
        }
    }

    #[test]
    fn test_parse_no_entities() {
        let text = "hello";
        let result = parse_predefined_entities(text.into()).unwrap();
        // this is the same slice
        assert!(std::ptr::eq(text, result.as_ref()));
    }

    #[test]
    fn test_serialize() {
        let text = "A & B";
        assert_eq!(serialize_predefined_entities(text.into()), "A &amp; B");
    }

    #[test]
    fn test_serialize_multiple() {
        let text = "&'><\"";
        assert_eq!(
            serialize_predefined_entities(text.into()),
            "&amp;&apos;&gt;&lt;&quot;"
        );
    }

    #[test]
    fn test_serialize_no_entities() {
        let text = "hello";
        let result = serialize_predefined_entities(text.into());
        // this is the same slice
        assert!(std::ptr::eq(text, result.as_ref()));
    }
}
