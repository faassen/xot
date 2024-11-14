use std::borrow::Cow;

use crate::error::{Error, ParseError};
use crate::output::Normalizer;

pub(crate) fn parse_text(content: Cow<str>) -> Result<Cow<str>, Error> {
    parse_content(content, false)
}

pub(crate) fn parse_attribute(content: Cow<str>) -> Result<Cow<str>, Error> {
    parse_content(content, true)
}

fn parse_content(content: Cow<str>, attribute: bool) -> Result<Cow<str>, Error> {
    let mut result = String::new();
    let mut chars = content.chars().peekable();
    let mut change = false;
    while let Some(c) = chars.next() {
        // https://www.w3.org/TR/xml/#sec-line-ends
        if c == '\r' {
            if chars.peek() == Some(&'\n') {
                // consume next char
                chars.next();
            }
            if !attribute {
                result.push('\n');
            } else {
                // https://www.w3.org/TR/xml/#AVNormalize
                result.push(' ');
            }
            change = true;
        } else if c == '&' {
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
                return Err(ParseError::UnclosedEntity(entity).into());
            }
            change = true;

            if let Some(entity) = entity.strip_prefix('#') {
                let first_char = entity
                    .chars()
                    .next()
                    .ok_or_else(|| ParseError::InvalidEntity(entity.to_string().into()))?;
                let code = if first_char == 'x' {
                    u32::from_str_radix(&entity[1..], 16)
                } else {
                    entity.parse::<u32>()
                };
                let code =
                    code.map_err(|_| ParseError::InvalidEntity(entity.to_string().into()))?;
                let c = std::char::from_u32(code)
                    .ok_or_else(|| ParseError::InvalidEntity(entity.to_string().into()))?;
                result.push(c);
            } else {
                match entity.as_str() {
                    "amp" => result.push('&'),
                    "apos" => result.push('\''),
                    "gt" => result.push('>'),
                    "lt" => result.push('<'),
                    "quot" => result.push('"'),
                    _ => return Err(ParseError::InvalidEntity(entity).into()),
                }
            }
        } else if attribute && (c == '\t' || c == '\n') {
            // https://www.w3.org/TR/xml/#AVNormalize
            // \r and \r\n already handled earlier
            result.push(' ');
            change = true;
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

pub(crate) fn serialize_text<'a, N: Normalizer>(
    content: Cow<'a, str>,
    normalizer: &N,
    unescaped_gt: bool,
) -> Cow<'a, str> {
    let mut result = String::new();
    let mut change = false;
    // if we had normalized_iter on the trait we avoid this string allocation
    let normalized_content = normalizer.normalize(content);

    for c in normalized_content.chars() {
        match c {
            '&' => {
                change = true;
                result.push_str("&amp;")
            }
            '<' => {
                change = true;
                result.push_str("&lt;")
            }
            '>' if !unescaped_gt => {
                change = true;
                result.push_str("&gt;")
            }
            '>' if unescaped_gt => {
                change = true;
                // take last two characters added to result
                let mut last_two = result.chars().rev().take(2);
                let last = last_two.next();
                if let Some(']') = last {
                    let last = last_two.next();
                    if let Some(']') = last {
                        // we are in the special ]]> sequence, so escape it
                        result.push_str("&gt;");
                        continue;
                    }
                }
                result.push('>');
            }
            _ => result.push(c),
        }
    }

    if !change {
        normalized_content
    } else {
        result.into()
    }
}

pub(crate) fn serialize_cdata<'a, N: Normalizer>(
    content: Cow<'a, str>,
    normalizer: &N,
) -> Cow<'a, str> {
    let mut result = String::new();
    result.push_str("<![CDATA[");
    // we write the content, watching for any possible sequence of "]]>"
    let mut closing_square_brackets_seen = 0;
    let normalized_content = normalizer.normalize(content);
    for c in normalized_content.chars() {
        match c {
            ']' => {
                if closing_square_brackets_seen < 2 {
                    closing_square_brackets_seen += 1;
                } else {
                    // if we're three, so write it and then start counting again
                    result.push(c);
                    // we are still at the critical junction
                    closing_square_brackets_seen = 2;
                }
            }
            '>' => {
                if closing_square_brackets_seen == 2 {
                    // we are the sequence
                    result.push_str("]]]]><![CDATA[>");
                } else {
                    // push any closing square brackets we've seen
                    for _ in 0..closing_square_brackets_seen {
                        result.push(']');
                    }
                    result.push(c);
                }
                closing_square_brackets_seen = 0;
            }
            _ => {
                // push any closing square brackets we've seen
                for _ in 0..closing_square_brackets_seen {
                    result.push(']');
                }
                closing_square_brackets_seen = 0;
                result.push(c)
            }
        }
    }
    // push any closing square brackets we've seen
    for _ in 0..closing_square_brackets_seen {
        result.push(']');
    }
    result.push_str("]]>");
    result.into()
}

pub(crate) fn serialize_attribute<'a, N: Normalizer>(
    content: Cow<'a, str>,
    normalizer: &N,
) -> Cow<'a, str> {
    let mut result = String::new();
    let mut change = false;
    let normalized_content = normalizer.normalize(content);
    for c in normalized_content.chars() {
        match c {
            '&' => {
                change = true;
                result.push_str("&amp;")
            }
            '<' => {
                change = true;
                result.push_str("&lt;")
            }
            '\'' => {
                change = true;
                result.push_str("&apos;")
            }
            '"' => {
                change = true;
                result.push_str("&quot;")
            }
            _ => result.push(c),
        }
    }

    if !change {
        normalized_content
    } else {
        result.into()
    }
}

#[cfg(test)]
mod tests {

    use crate::output::NoopNormalizer;

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
        if let Err(Error::Parse(ParseError::InvalidEntity(entity))) = err {
            assert_eq!(entity, "unknown");
        } else {
            unreachable!();
        }
    }

    #[test]
    fn test_parse_unfinished_entity() {
        let text = "&amp";
        let err = parse_text(text.into());
        if let Err(Error::Parse(ParseError::UnclosedEntity(entity))) = err {
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
    fn test_parse_newline_r() {
        let text = "A \r B";
        assert_eq!(parse_text(text.into()).unwrap(), "A \n B");
    }

    #[test]
    fn test_parse_newline_rn() {
        let text = "A \r\n B";
        assert_eq!(parse_text(text.into()).unwrap(), "A \n B");
    }

    #[test]
    fn test_do_not_normalize_text_tab() {
        let text = "A \t B";
        assert_eq!(parse_text(text.into()).unwrap(), "A \t B");
    }

    #[test]
    fn test_do_not_normalize_text_newline() {
        let text = "A \n B";
        assert_eq!(parse_text(text.into()).unwrap(), "A \n B");
    }

    #[test]
    fn test_normalize_attribute_tab() {
        let text = "A \t B";
        assert_eq!(parse_attribute(text.into()).unwrap(), "A   B");
    }

    #[test]
    fn test_normalize_attribute_r_newline() {
        let text = "A \r B";
        assert_eq!(parse_attribute(text.into()).unwrap(), "A   B");
    }

    #[test]
    fn test_normalize_attribute_rn_newline() {
        let text = "A \r\n B";
        assert_eq!(parse_attribute(text.into()).unwrap(), "A   B");
    }

    #[test]
    fn test_normalize_attribute_newline() {
        let text = "A \n B";
        assert_eq!(parse_attribute(text.into()).unwrap(), "A   B");
    }

    #[test]
    fn test_serialize_text() {
        let text = "A & B";
        assert_eq!(
            serialize_text(text.into(), &NoopNormalizer, false),
            "A &amp; B"
        );
    }

    #[test]
    fn test_serialize_text_multiple() {
        let text = "&<'\">";
        assert_eq!(
            serialize_text(text.into(), &NoopNormalizer, false),
            "&amp;&lt;'\"&gt;"
        );
    }

    #[test]
    fn test_serialize_text_gt_escaped() {
        let text = ">";
        assert_eq!(serialize_text(text.into(), &NoopNormalizer, false), "&gt;");
    }

    #[test]
    fn test_serialize_text_gt_unescaped() {
        let text = ">";
        assert_eq!(serialize_text(text.into(), &NoopNormalizer, true), ">");
    }

    #[test]
    fn test_serialize_text_like_cdata_section_close_delimiter() {
        let text = "]]>";
        assert_eq!(
            serialize_text(text.into(), &NoopNormalizer, false),
            "]]&gt;"
        );
    }

    #[test]
    fn test_serialize_text_like_cdata_section_close_delimiter_gt_unescaped() {
        let text = "]]>";
        // this needs to happen even if the gt is asked to be unescaped
        assert_eq!(serialize_text(text.into(), &NoopNormalizer, true), "]]&gt;");
    }

    #[test]
    fn test_serialize_text_like_cdata_section_close_delimiter_split() {
        let text = "]]extra>";
        assert_eq!(
            serialize_text(text.into(), &NoopNormalizer, true),
            "]]extra>"
        );
    }
    #[test]
    fn test_serialize_text_like_cdata_section_close_delimiter_prefix() {
        let text = "extra]]>";
        assert_eq!(
            serialize_text(text.into(), &NoopNormalizer, true),
            "extra]]&gt;"
        );
    }

    #[test]
    fn test_serialize_text_like_cdata_section_close_delimiter_postfix() {
        let text = "]]>extra";
        assert_eq!(
            serialize_text(text.into(), &NoopNormalizer, true),
            "]]&gt;extra"
        );
    }

    #[test]
    fn test_serialize_text_no_entities() {
        let text = "hello";
        let result = serialize_text(text.into(), &NoopNormalizer, false);
        // this is the same slice
        assert!(std::ptr::eq(text, result.as_ref()));
    }

    #[test]
    fn test_serialize_attribute() {
        let text = "A & B";
        assert_eq!(
            serialize_attribute(text.into(), &NoopNormalizer),
            "A &amp; B"
        );
    }

    #[test]
    fn test_serialize_attribute_multiple_single() {
        let text = "&<'";
        assert_eq!(
            serialize_attribute(text.into(), &NoopNormalizer),
            "&amp;&lt;&apos;"
        );
    }

    #[test]
    fn test_serialize_attribute_multiple_double() {
        let text = "&<\"";
        assert_eq!(
            serialize_attribute(text.into(), &NoopNormalizer),
            "&amp;&lt;&quot;"
        );
    }

    #[test]
    fn test_serialize_attribute_no_entities() {
        let text = "hello";
        let result = serialize_attribute(text.into(), &NoopNormalizer);
        // this is the same slice
        assert!(std::ptr::eq(text, result.as_ref()));
    }

    #[test]
    fn test_parse_character_hex_entity() {
        let text = "A &#x26; B";
        assert_eq!(parse_text(text.into()).unwrap(), "A & B");
    }

    #[test]
    fn test_parse_character_decimal_entity() {
        let text = "A &#38; B";
        assert_eq!(parse_text(text.into()).unwrap(), "A & B");
    }

    #[test]
    fn test_parse_character_empty_entity() {
        let text = "A &#; B";
        assert!(parse_text(text.into()).is_err());
    }

    #[test]
    fn test_parse_character_empty_hex_entity() {
        let text = "A &x#; B";
        assert!(parse_text(text.into()).is_err());
    }

    #[test]
    fn test_parse_character_broken_hex_entity() {
        let text = "A &xflub#; B";
        assert!(parse_text(text.into()).is_err());
    }

    #[test]
    fn test_serialize_cdata_simple() {
        let text = "hello";
        assert_eq!(
            serialize_cdata(text.into(), &NoopNormalizer),
            "<![CDATA[hello]]>"
        );
    }

    #[test]
    fn test_serialize_cdata_end_sequence() {
        let text = "hello]]>world";
        assert_eq!(
            serialize_cdata(text.into(), &NoopNormalizer),
            "<![CDATA[hello]]]]><![CDATA[>world]]>"
        );
    }

    #[test]
    fn test_serialize_cdata_two_square_brackets() {
        let text = "hello]]world";
        assert_eq!(
            serialize_cdata(text.into(), &NoopNormalizer),
            "<![CDATA[hello]]world]]>"
        );
    }

    // two square brackets at the end
    #[test]
    fn test_serialize_cdata_two_square_brackets_end() {
        let text = "hello]]";
        assert_eq!(
            serialize_cdata(text.into(), &NoopNormalizer),
            "<![CDATA[hello]]]]>"
        );
    }

    // greater than sign by itself
    #[test]
    fn test_serialize_cdata_greater_than() {
        let text = ">";
        assert_eq!(
            serialize_cdata(text.into(), &NoopNormalizer),
            "<![CDATA[>]]>"
        );
    }

    // special sequence by itself
    #[test]
    fn test_serialize_cdata_special_sequence() {
        let text = "]]>";
        assert_eq!(
            serialize_cdata(text.into(), &NoopNormalizer),
            "<![CDATA[]]]]><![CDATA[>]]>"
        );
    }

    // three square brackets
    #[test]
    fn test_serialize_cdata_three_square_brackets() {
        let text = "hello]]]world";
        assert_eq!(
            serialize_cdata(text.into(), &NoopNormalizer),
            "<![CDATA[hello]]]world]]>"
        );
    }

    // three square brackets ending in special sequence
    #[test]
    fn test_serialize_cdata_three_square_brackets_end_sequence() {
        let text = "hello]]]>world";
        assert_eq!(
            serialize_cdata(text.into(), &NoopNormalizer),
            "<![CDATA[hello]]]]]><![CDATA[>world]]>"
        );
    }
}
