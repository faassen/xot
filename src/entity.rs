use std::borrow::Cow;

use crate::error::Error;

pub(crate) fn parse_text(content: Cow<str>) -> Result<Cow<str>, Error> {
    let mut result = String::new();
    let mut chars = content.chars();
    let mut change = false;
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
            change = true;
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

    if !change {
        Ok(content)
    } else {
        Ok(result.into())
    }
}

pub(crate) fn serialize_text(content: Cow<str>) -> Cow<str> {
    let mut result = String::new();
    let mut change = false;
    for c in content.chars() {
        match c {
            '&' => {
                change = true;
                result.push_str("&amp;")
            }
            '\'' => {
                change = true;
                result.push_str("&apos;")
            }
            '>' => {
                change = true;
                result.push_str("&gt;")
            }
            '<' => {
                change = true;
                result.push_str("&lt;")
            }
            '"' => {
                change = true;
                result.push_str("&quot;")
            }
            _ => result.push(c),
        }
    }

    if !change {
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
        assert_eq!(parse_text(text.into()).unwrap(), "A & B");
    }

    #[test]
    fn test_parse_multiple() {
        let text = "&amp;&apos;&gt;&lt;&quot;";
        assert_eq!(parse_text(text.into()).unwrap(), "&'><\"");
    }

    #[test]
    fn test_parse_unknown_entity() {
        let text = "&unknown;";
        let err = parse_text(text.into());
        if let Err(Error::InvalidEntity(entity)) = err {
            assert_eq!(entity, "unknown");
        } else {
            unreachable!();
        }
    }

    #[test]
    fn test_parse_unfinished_entity() {
        let text = "&amp";
        let err = parse_text(text.into());
        if let Err(Error::UnclosedEntity(entity)) = err {
            assert_eq!(entity, "amp");
        } else {
            unreachable!();
        }
    }

    #[test]
    fn test_parse_no_entities() {
        let text = "hello";
        let result = parse_text(text.into()).unwrap();
        // this is the same slice
        assert!(std::ptr::eq(text, result.as_ref()));
    }

    #[test]
    fn test_serialize() {
        let text = "A & B";
        assert_eq!(serialize_text(text.into()), "A &amp; B");
    }

    #[test]
    fn test_serialize_multiple() {
        let text = "&'><\"";
        assert_eq!(serialize_text(text.into()), "&amp;&apos;&gt;&lt;&quot;");
    }

    #[test]
    fn test_serialize_no_entities() {
        let text = "hello";
        let result = serialize_text(text.into());
        // this is the same slice
        assert!(std::ptr::eq(text, result.as_ref()));
    }
}
