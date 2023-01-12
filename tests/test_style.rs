use xot::{Error, Node, OutputToken, Pretty, SerializationData, Serializer, Xot};

#[test]
fn test_style() -> Result<(), Error> {
    // we have XML with in it particular `style` wrappers around text nodes
    // We want to render this XML to HTML (so the XML is escaped) with the
    // style wrappers removed and replaced by a <span>. We want the resulting
    // rendered XML to be pretty printed.
    //
    // We do this by using the advanced serialization features
    let mut xot = Xot::new();

    let root = xot.parse("<a><b><style>foo</style></b></a>")?;
    // expected:
    // <a>
    //  <b><span>foo</span></b>
    // </a>
    // because foo is text content, and thus span shouldn't be indented or affected
    // we currently determine text content by using the xot structure, but
    // we should instead peek ahead in the stream to determine a text node
    // so the stream should be ahead by all children of the current node
    // This way Pretty doesn't need access to xot anymore.
    let name_style = xot.add_name("style");

    let mut serializer = xot.serializer(root);
    let extra_prefixes = xot::Prefixes::new();
    let output_tokens = xot.output_tokens(root, &extra_prefixes);
    let mut pretty = xot.pretty();

    #[derive(Debug, PartialEq, Eq)]
    enum Style {
        Start,
        End,
        Text(String),
    }

    let mut output = Vec::new();

    use OutputToken::*;

    fn render_normal(
        indentation: usize,
        newline: bool,
        rendered: SerializationData,
        output: &mut Vec<Style>,
    ) {
        if indentation > 0 {
            output.push(Style::Text(" ".repeat(indentation * 2)));
        }
        if rendered.space {
            output.push(Style::Text(" ".to_string()));
        }
        output.push(Style::Text(rendered.text));
        if newline {
            output.push(Style::Text("\n".to_string()));
        }
    }

    for (node, output_token) in output_tokens {
        match output_token {
            StartTagOpen(element) => {
                if element.name() == name_style {
                    output.push(Style::Start);
                } else {
                    let (indentation, newline) = pretty.prettify(node, &output_token);
                    let rendered = serializer.render(node, output_token)?;
                    render_normal(indentation, newline, rendered, &mut output);
                }
            }
            StartTagClose(element) => {
                if element.name() == name_style {
                    // do nothing
                } else {
                    let (indentation, newline) = pretty.prettify(node, &output_token);
                    let rendered = serializer.render(node, output_token)?;
                    render_normal(indentation, newline, rendered, &mut output);
                }
            }
            EndTag(element) => {
                if element.name() == name_style {
                    output.push(Style::End);
                } else {
                    let (indentation, newline) = pretty.prettify(node, &output_token);
                    let rendered = serializer.render(node, output_token)?;
                    render_normal(indentation, newline, rendered, &mut output);
                }
            }
            _ => {
                let (indentation, newline) = pretty.prettify(node, &output_token);
                let rendered = serializer.render(node, output_token)?;
                render_normal(indentation, newline, rendered, &mut output);
            }
        }
    }
    let output = output
        .iter()
        .map(|style| match style {
            Style::Start => "<span>".to_string(),
            Style::End => "</span>".to_string(),
            Style::Text(text) => text.to_string(),
        })
        .collect::<Vec<_>>()
        .join("");

    assert_eq!(output, "");
    Ok(())
}
