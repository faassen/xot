use xot::{Error, Node, OutputToken, Pretty, Serializer, Xot};

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

    let name_style = xot.add_name("style");

    let serializer = xot.serializer(root);
    let output_tokens = serializer.output_tokens();
    let pretty = xot.pretty();

    #[derive(Debug, PartialEq, Eq)]
    enum Style {
        Start,
        End,
        Text(String),
    }

    let output = Vec::new();

    use OutputToken::*;

    fn render_normal(
        serializer: &mut Serializer,
        pretty: &mut Pretty,
        node: Node,
        output_token: &OutputToken,
        output: &mut Vec<Style>,
    ) -> Result<(), Error> {
        let (indentation, newline) = pretty.prettify(node, output_token);
        let rendered = serializer.render(node, output_token)?;
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
        Ok(())
    }

    for (node, output_token) in output_tokens {
        match output_token {
            StartTagOpen(element) => {
                if element.name() == name_style {
                    output.push(Style::Start);
                } else {
                    render_normal(
                        &mut serializer,
                        &mut pretty,
                        node,
                        &output_token,
                        &mut output,
                    );
                }
            }
            EndTag(element) => {
                if element.name() == name_style {
                    output.push(Style::End);
                } else {
                    render_normal(
                        &mut serializer,
                        &mut pretty,
                        node,
                        &output_token,
                        &mut output,
                    );
                }
            }
            _ => {
                render_normal(
                    &mut serializer,
                    &mut pretty,
                    node,
                    &output_token,
                    &mut output,
                );
            }
        }
    }

    assert_eq!(
        output,
        vec![
            Style::Text("<a>".to_string()),
            Style::Text("\n".to_string()),
            Style::Text("  ".to_string()),
            Style::Text("<b>".to_string()),
            Style::Text("\n".to_string()),
            Style::Start,
            Style::Text("foo".to_string()),
            Style::End,
            Style::Text("\n".to_string()),
            Style::Text("  ".to_string()),
            Style::Text("</b>".to_string()),
            Style::Text("\n".to_string()),
            Style::Text("</a>".to_string()),
            Style::Text("\n".to_string()),
        ]
    );
    Ok(())
}
