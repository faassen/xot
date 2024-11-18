use xot::{
    output::{Output, TokenSerializeParameters},
    ParseError, Xot,
};

#[test]
fn test_style() -> Result<(), ParseError> {
    let mut xot = Xot::new();

    let root = xot.parse("<a><b><style>foo</style></b></a>")?;

    // We first take away the style marker from the tree, and note where it was
    // we do this by hand here but a general procedure could be devised
    let a = xot.first_child(root).unwrap();
    let b = xot.first_child(a).unwrap();
    let style = xot.first_child(b).unwrap();
    let text = xot.first_child(style).unwrap();
    // now we unwrap the style
    xot.element_unwrap(style).unwrap();

    let suppress = vec![];
    let pretty_tokens = xot.pretty_tokens(
        root,
        TokenSerializeParameters::default(),
        &suppress,
        xot::output::NoopNormalizer,
    );

    #[derive(Debug, PartialEq, Eq)]
    enum Style {
        Start,
        End,
        Text(String),
    }

    let mut result = Vec::new();

    for (node, _output, token) in pretty_tokens {
        if token.indentation > 0 {
            result.push(Style::Text(" ".repeat(token.indentation * 2)));
        }
        if token.space {
            result.push(Style::Text(" ".to_string()));
        }
        if node == text {
            result.push(Style::Start);
        }
        result.push(Style::Text(token.text));
        if node == text {
            result.push(Style::End);
        }
        if token.newline {
            result.push(Style::Text("\n".to_string()));
        }
    }

    let result = result
        .iter()
        .map(|style| match style {
            Style::Start => "<span>".to_string(),
            Style::End => "</span>".to_string(),
            Style::Text(text) => text.to_string(),
        })
        .collect::<Vec<_>>()
        .join("");

    assert_eq!(result, "<a>\n  <b><span>foo</span></b>\n</a>\n");
    Ok(())
}

#[test]
fn test_style_element() -> Result<(), ParseError> {
    let mut xot = Xot::new();

    let root = xot.parse("<a><b>foo</b></a>")?;
    // the b element needs to be wrapped with spans
    let a = xot.first_child(root).unwrap();
    let b = xot.first_child(a).unwrap();

    let suppress = vec![];

    let pretty_tokens = xot.pretty_tokens(
        root,
        TokenSerializeParameters::default(),
        &suppress,
        xot::output::NoopNormalizer,
    );

    #[derive(Debug, PartialEq, Eq)]
    enum Style {
        Start,
        End,
        Text(String),
    }

    let mut result = Vec::new();

    for (node, output, rendered) in pretty_tokens {
        if rendered.indentation > 0 {
            result.push(Style::Text(" ".repeat(rendered.indentation * 2)));
        }
        if rendered.space {
            result.push(Style::Text(" ".to_string()));
        }
        if node == b {
            if let Output::StartTagOpen(_) = output {
                result.push(Style::Start);
            }
        }

        result.push(Style::Text(rendered.text));

        if node == b {
            if let Output::EndTag(_) = output {
                result.push(Style::End);
            }
        }
        if rendered.newline {
            result.push(Style::Text("\n".to_string()));
        }
    }

    let result = result
        .iter()
        .map(|style| match style {
            Style::Start => "<span>".to_string(),
            Style::End => "</span>".to_string(),
            Style::Text(text) => text.to_string(),
        })
        .collect::<Vec<_>>()
        .join("");

    assert_eq!(result, "<a>\n  <span><b>foo</b></span>\n</a>\n");
    Ok(())
}

#[test]
fn test_style_attribute() -> Result<(), ParseError> {
    let mut xot = Xot::new();

    let root = xot.parse(r#"<doc a="A" b="B"/>"#)?;
    let name_a = xot.add_name("a");

    let suppress = vec![];
    let pretty_tokens = xot.pretty_tokens(
        root,
        TokenSerializeParameters::default(),
        &suppress,
        xot::output::NoopNormalizer,
    );

    #[derive(Debug, PartialEq, Eq)]
    enum Style {
        Start,
        End,
        Text(String),
    }

    let mut result = Vec::new();

    for (_node, output, rendered) in pretty_tokens {
        if rendered.indentation > 0 {
            result.push(Style::Text(" ".repeat(rendered.indentation * 2)));
        }
        if rendered.space {
            result.push(Style::Text(" ".to_string()));
        }

        if let Output::Attribute(name, _value) = output {
            if name == name_a {
                result.push(Style::Start);
            }
        }
        result.push(Style::Text(rendered.text));
        if let Output::Attribute(name, _value) = output {
            if name == name_a {
                result.push(Style::End);
            }
        }

        if rendered.newline {
            result.push(Style::Text("\n".to_string()));
        }
    }

    let result = result
        .iter()
        .map(|style| match style {
            Style::Start => "<span>".to_string(),
            Style::End => "</span>".to_string(),
            Style::Text(text) => text.to_string(),
        })
        .collect::<Vec<_>>()
        .join("");

    assert_eq!(
        result,
        r#"<doc <span>a="A"</span> b="B"/>
"#
    );
    Ok(())
}
