use ahash::{HashSet, HashSetExt};

use crate::id::NameId;
use crate::xotdata::Xot;
use crate::NamespaceId;

// used to determine whether something is a HTML 5 element
const XHTML_NS: &str = "https://www.w3.org/1999/xhtml";
const MATHML_NS: &str = "http://www.w3.org/1998/Math/MathML";
const SVG_NS: &str = "http://www.w3.org/2000/svg";

#[derive(Debug)]
pub(crate) struct Html5Elements {
    xhtml_namespace_id: NamespaceId,
    mathml_namespace_id: NamespaceId,
    svg_namespace_id: NamespaceId,
    pub(crate) html5_names: HtmlNames,
    pub(crate) phrasing_content_names: HtmlNames,
    pub(crate) void_names: HtmlNames,
    pub(crate) formatted_names: HtmlNames,
    pub(crate) no_escape_names: HtmlNames,
}

#[derive(Debug)]
pub(crate) struct HtmlNames {
    xhtml_namespace_id: NamespaceId,
    ids: HashSet<NameId>,
    names: HashSet<String>,
}

impl HtmlNames {
    fn new(xot: &mut Xot, xhtml_namespace_id: NamespaceId, names: &[&str]) -> Self {
        let mut ids = HashSet::new();
        for name in names {
            // lowercase names, no namespace
            ids.insert(xot.add_name_ns(name, xot.no_namespace()));
            // uppercase names, no namespace
            ids.insert(xot.add_name_ns(&name.to_ascii_uppercase(), xot.no_namespace()));
            // lowercase names, XHTML namespace
            ids.insert(xot.add_name_ns(name, xhtml_namespace_id));
            // uppercase names, XHTML namespace
            ids.insert(xot.add_name_ns(&name.to_ascii_uppercase(), xhtml_namespace_id));
        }
        Self {
            xhtml_namespace_id,
            ids,
            names: names.iter().map(|name| name.to_string()).collect(),
        }
    }

    pub(crate) fn is_html_element(&self, xot: &Xot, name_id: NameId) -> bool {
        let namespace = xot.namespace_for_name(name_id);
        namespace == self.xhtml_namespace_id || namespace == xot.no_namespace()
    }

    pub(crate) fn matches(&self, xot: &Xot, name_id: NameId) -> bool {
        // if we match any of the known ids, we're done right away
        if self.ids.contains(&name_id) {
            return true;
        }
        // if this is not an HTML element, we know it isn't one of the HTML names
        if !self.is_html_element(xot, name_id) {
            return false;
        }
        // otherwise, we do a case-insensitive lookup of the local name
        let name = xot.local_name_str(name_id);
        // now lowercase the name and look it up
        let name = name.to_ascii_lowercase();
        self.names.contains(&name)
    }
}

impl Html5Elements {
    pub(crate) fn new(xot: &mut Xot) -> Self {
        let xhtml_namespace_id = xot.add_namespace(XHTML_NS);
        let mathml_namespace_id = xot.add_namespace(MATHML_NS);
        let svg_namespace_id = xot.add_namespace(SVG_NS);
        let html5_names = [
            "a",
            "abbr",
            "address",
            "area",
            "article",
            "aside",
            "audio",
            "b",
            "base",
            "bdi",
            "bdo",
            "blockquote",
            "body",
            "br",
            "button",
            "canvas",
            "caption",
            "cite",
            "code",
            "col",
            "colgroup",
            "command",
            "datalist",
            "dd",
            "del",
            "details",
            "dfn",
            "div",
            "dl",
            "dt",
            "em",
            "embed",
            "fieldset",
            "figcaption",
            "figure",
            "footer",
            "form",
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
            "head",
            "header",
            "hgroup",
            "hr",
            "html",
            "i",
            "iframe",
            "img",
            "input",
            "ins",
            "kbd",
            "keygen",
            "label",
            "legend",
            "li",
            "link",
            "map",
            "mark",
            "math",
            "menu",
            "meta",
            "meter",
            "nav",
            "noscript",
            "object",
            "ol",
            "optgroup",
            "option",
            "output",
            "p",
            "param",
            "pre",
            "progress",
            "q",
            "rp",
            "rt",
            "ruby",
            "s",
            "samp",
            "script",
            "section",
            "select",
            "small",
            "source",
            "span",
            "strong",
            "style",
            "sub",
            "summary",
            "sup",
            "table",
            "tbody",
            "td",
            "template",
            "textarea",
            "tfoot",
            "th",
            "thead",
            "time",
            "title",
            "tr",
            "track",
            "u",
            "ul",
            "var",
            "video",
            "wbr",
        ];
        let html5_names = HtmlNames::new(xot, xhtml_namespace_id, &html5_names);

        let void_names = [
            "area", "base", "br", "col", "embed", "hr", "img", "input", "keygen", "link", "meta",
            "param", "source", "track", "wbr",
            // extra elements not in the HTML5 spec but null in HTML 4
            "basefont", "frame", "isindex",
        ];

        let void_names = HtmlNames::new(xot, xhtml_namespace_id, &void_names);

        let phrasing_content_names = [
            "a", "abbr", "area", "audio", "b", "bdi", "bdo", "br", "button", "canvas", "cite",
            "code", "command", "datalist", "del", "dfn", "em", "embed", "i", "iframe", "img",
            "input", "ins", "kbd", "keygen", "label", "map", "mark", "math", "meter", "noscript",
            "object", "output", "progress", "q", "ruby", "s", "samp", "script", "select", "small",
            "span", "strong", "sub", "sup", "svg", "textarea", "time", "u", "var", "video", "wbr",
        ];
        let phrasing_content_names =
            HtmlNames::new(xot, xhtml_namespace_id, &phrasing_content_names);

        let formatted_names = ["pre", "script", "style", "title", "textarea"];
        let formatted_names = HtmlNames::new(xot, xhtml_namespace_id, &formatted_names);

        let no_escape_names = ["script", "style"];
        let no_escape_names = HtmlNames::new(xot, xhtml_namespace_id, &no_escape_names);
        Self {
            html5_names,
            xhtml_namespace_id,
            mathml_namespace_id,
            svg_namespace_id,
            void_names,
            phrasing_content_names,
            formatted_names,
            no_escape_names,
        }
    }

    pub(crate) fn is_html_element(&self, xot: &Xot, name_id: NameId) -> bool {
        let namespace = xot.namespace_for_name(name_id);
        self.is_html_namespace(xot, namespace)
    }

    pub(crate) fn must_be_serialized_unprefixed(&self, namespace: NamespaceId) -> bool {
        namespace == self.xhtml_namespace_id
            || namespace == self.mathml_namespace_id
            || namespace == self.svg_namespace_id
    }

    pub(crate) fn is_html_namespace(&self, xot: &Xot, namespace_id: NamespaceId) -> bool {
        namespace_id == self.xhtml_namespace_id || namespace_id == xot.no_namespace()
    }
}
