use markdown_it::{generics::inline::full_link, *};
use phosphor_leptos::{ARROW_SQUARE_OUT, IconWeight};

/// Same as `markdown_it::plugins::cmark::inline::link::Link`, but adds icon indicating that
/// it is external link.
#[derive(Debug)]
pub struct ExternalLink {
    pub url: String,
    pub title: Option<String>,
}

impl NodeValue for ExternalLink {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        let mut attrs = node.attrs.clone();
        attrs.push(("href", self.url.clone()));

        if let Some(title) = &self.title {
            attrs.push(("title", title.clone()));
        }

        fmt.open("a", &attrs);
        fmt.contents(&node.children);

        // Icon to indicate external link
        let svg_attrs = [
            ("xmlns", "http://www.w3.org/2000/svg".to_string()),
            ("width", "1em".to_string()),
            ("height", "1em".to_string()),
            ("fill", "currentColor".to_string()),
            ("viewBox", "0 0 256 256".to_string()),
            ("style", "display: inline; margin:4px;".to_string()),
        ];
        fmt.open("svg", &svg_attrs);
        fmt.text_raw(ARROW_SQUARE_OUT.get(IconWeight::Regular));
        fmt.close("svg");

        fmt.close("a");
    }
}

pub fn add(md: &mut MarkdownIt) {
    full_link::add::<false>(md, |href, title| {
        Node::new(ExternalLink {
            url: href.unwrap_or_default(),
            title,
        })
    });
}
