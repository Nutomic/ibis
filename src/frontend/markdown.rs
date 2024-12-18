use katex;
use markdown_it::{
    parser::inline::{InlineRule, InlineState},
    plugins::cmark::block::{heading::ATXHeading, lheading::SetextHeader},
    MarkdownIt,
    Node,
    NodeValue,
    Renderer,
};
use once_cell::sync::OnceCell;

pub fn render_markdown(text: &str) -> String {
    static INSTANCE: OnceCell<MarkdownIt> = OnceCell::new();
    let mut parsed = INSTANCE.get_or_init(markdown_parser).parse(text);

    // Make markdown headings one level smaller, so that h1 becomes h2 etc, and markdown titles
    // are smaller than page title.
    parsed.walk_mut(|node, _| {
        if let Some(heading) = node.cast_mut::<ATXHeading>() {
            heading.level += 1;
        }
        if let Some(heading) = node.cast_mut::<SetextHeader>() {
            heading.level += 1;
        }
    });
    parsed.render()
}

fn markdown_parser() -> MarkdownIt {
    let mut parser = MarkdownIt::new();
    let p = &mut parser;
    {
        // Markdown-it inline core features. Image is disabled to prevent embedding external
        // images. Later we need to add proper image support using pictrs.
        use markdown_it::plugins::cmark::inline::*;
        newline::add(p);
        escape::add(p);
        backticks::add(p);
        emphasis::add(p);
        link::add(p);
        image::add(p);
        autolink::add(p);
        entity::add(p);
    }

    {
        // Markdown-it block core features. Unchanged from defaults.
        use markdown_it::plugins::cmark::block::*;
        code::add(p);
        fence::add(p);
        blockquote::add(p);
        hr::add(p);
        list::add(p);
        reference::add(p);
        heading::add(p);
        lheading::add(p);
        paragraph::add(p);
    }

    {
        // Some of the extras from markdown-it, others are intentionally excluded.
        use markdown_it::plugins::extra::*;
        strikethrough::add(p);
        tables::add(p);
        typographer::add(p);
    }

    // Extensions from various authors
    markdown_it_heading_anchors::add(p);
    markdown_it_block_spoiler::add(p);
    markdown_it_footnote::add(p);
    markdown_it_sub::add(p);
    markdown_it_sup::add(p);

    // Ibis custom extensions
    parser.inline.add_rule::<ArticleLinkScanner>();
    parser.inline.add_rule::<MathEquationScanner>();

    parser
}

#[derive(Debug)]
pub struct ArticleLink {
    label: String,
    title: String,
    domain: String,
}

// This defines how your custom node should be rendered.
impl NodeValue for ArticleLink {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        let mut attrs = node.attrs.clone();

        let link = format!("/article/{}@{}", self.title, self.domain);
        attrs.push(("href", link));

        fmt.open("a", &attrs);
        fmt.text(&self.label);
        fmt.close("a");
    }
}

struct ArticleLinkScanner;

impl InlineRule for ArticleLinkScanner {
    const MARKER: char = '[';

    /// Find `[[Title@example.com]], return the position and split title/domain.
    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        let input = &state.src[state.pos..state.pos_max];
        if !input.starts_with("[[") {
            return None;
        }
        const SEPARATOR_LENGTH: usize = 2;

        input.find("]]").and_then(|length| {
            let start = state.pos + SEPARATOR_LENGTH;
            let i = start + length - SEPARATOR_LENGTH;
            let content = &state.src[start..i];
            content.split_once('@').map(|(title, domain)| {
                // Handle custom link label if provided, otherwise use title as label
                let (domain, label) = domain.split_once('|').unwrap_or((&domain, &title));
                let node = Node::new(ArticleLink {
                    label: label.to_string(),
                    title: title.to_string(),
                    domain: domain.to_string(),
                });
                (node, length + SEPARATOR_LENGTH)
            })
        })
    }
}

#[derive(Debug)]
pub struct MathEquation {
    equation: String,
    display_mode: bool,
}

impl NodeValue for MathEquation {
    fn render(&self, _node: &Node, fmt: &mut dyn Renderer) {
        let opts = katex::Opts::builder()
            .throw_on_error(false)
            .display_mode(self.display_mode)
            .build()
            .ok();
        let katex_equation = opts.and_then(|o| katex::render_with_opts(&self.equation, o).ok());
        fmt.text_raw(katex_equation.as_ref().unwrap_or(&self.equation))
    }
}

struct MathEquationScanner;

impl InlineRule for MathEquationScanner {
    const MARKER: char = '$';

    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        let input = &state.src[state.pos..state.pos_max];
        if !input.starts_with("$$") {
            return None;
        }
        let mut display_mode = false;
        if input.starts_with("$$\n") || input.starts_with("$$ ") {
            display_mode = true;
        }
        const SEPARATOR_LENGTH: usize = 2;

        input[SEPARATOR_LENGTH - 1..].find("$$").map(|length| {
            let start = state.pos + SEPARATOR_LENGTH;
            let i = start + length - SEPARATOR_LENGTH + 1;
            if start > i {
                return None;
            }
            let content = &state.src[start..i];
            let node = Node::new(MathEquation {
                equation: content.to_string(),
                display_mode,
            });
            Some((node, length + SEPARATOR_LENGTH + 1))
        })?
    }
}

#[test]
fn test_markdown_article_link() {
    let parser = markdown_parser();
    let plain = parser.parse("[[Title@example.com]]").render();
    assert_eq!(
        "<p><a href=\"/article/Title@example.com\">Title</a></p>\n",
        plain
    );

    let with_label = parser
        .parse("[[Title@example.com|Example Article]]")
        .render();
    assert_eq!(
        "<p><a href=\"/article/Title@example.com\">Example Article</a></p>\n",
        with_label
    );
}

#[test]
#[expect(clippy::unwrap_used)]
fn test_markdown_equation_katex() {
    let parser = markdown_parser();
    let rendered = parser
        .parse("here is a math equation: $$E=mc^2$$. Pretty cool, right?")
        .render();
    assert_eq!(
        "<p>here is a math equation: ".to_owned()
            + &katex::render("E=mc^2").unwrap()
            + ". Pretty cool, right?</p>\n",
        rendered
    );
}
