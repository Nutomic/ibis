use katex;
use markdown_it::{
    parser::inline::{InlineRule, InlineState},
    parser::core::CoreRule,
    plugins::cmark::block::{heading::ATXHeading, lheading::SetextHeader},
    MarkdownIt,
    Node,
    NodeValue,
    Renderer,
};
use once_cell::sync::OnceCell;
use github_slugger::Slugger;

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
    parser.inline.add_rule::<TocMarkerScanner>();
    parser.add_rule::<TocScanner>();

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
                let (domain, label) = domain.split_once('|').unwrap_or((domain, title));
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



// TOC SECTION

#[derive(Debug, Default, Clone)]
pub struct Toc{
    entries: Vec<TocEntry>,
}

impl Toc {
    fn count_entries_with_level(&self, level: u8) -> usize {
        self.entries.iter().filter(|e| e.level == level).count()
    }

    fn print_entry(&self, fmt: &mut dyn Renderer){
            self
            .entries
            .iter()
            .for_each( |entry| {
                    fmt.open("ul", &[]);
                    fmt.open("li", &[]);
                    fmt.open("a", &[("href", "#".to_owned() + &entry.id)]);
                    
                    fmt.text(&format!("{} {}", entry.sec_number, entry.name));

                    fmt.close("a");

                    entry.children.print_entry(fmt);

                    fmt.close("li");
                    fmt.close("ul");
            });
        }
}


impl NodeValue for Toc {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        let mut attrs = node.attrs.clone();

        attrs.push(("class", "float-right mr-20 w-80 menu rounded-box".into()));

        fmt.open("div", &attrs);

        fmt.open("li", &[("class", "menu-title".into())]);
        fmt.text("Table of Contents");
        fmt.close("li");

        self.print_entry(fmt);

        fmt.close("div");
    }
}

#[derive(Debug, Clone)]
struct TocEntry {
    level: u8,
    sec_number: String,
    name: String,
    id: String,
    children: Toc,
}

#[derive(Default)]
struct TocBuilder {
    top_level: Toc,
    chain: Vec<TocEntry>,
}

impl TocBuilder {
    fn new() -> TocBuilder {
        TocBuilder::default()
    }

    fn into_toc(mut self) -> Toc {
        self.fold_until(0);
        self.top_level
    }

    fn fold_until(&mut self, level: u8) {
        let mut this = None;
        loop {
            match self.chain.pop() {
                Some(mut next) => {
                    next.children.entries.extend(this);
                    if next.level < level {
                        self.chain.push(next);
                        return;
                    } else {
                        this = Some(next);
                    }
                }
                None => {
                    self.top_level.entries.extend(this);
                    return;
                }
            }
        }
    }

    fn push(&mut self, level: u8, name: String, id: String) -> &str {
        debug_assert!(level >= 1);

        self.fold_until(level);

        let mut sec_number;
        {
            let toc = match self.chain.last() {
                None => {
                    sec_number = String::new();
                    &self.top_level
                }
                Some(entry) => {
                    sec_number = entry.sec_number.clone();
                    sec_number.push('.');
                    &entry.children
                }
            };
            let number = toc.count_entries_with_level(level);
            sec_number.push_str(&(number + 1).to_string())
        }

        self.chain.push(TocEntry {
            level,
            name,
            sec_number,
            id,
            children: Toc {
                entries: Vec::new(),
            },
        });

        let just_inserted = self.chain.last_mut().unwrap();
        &just_inserted.sec_number
    }
}

struct TocScanner;

impl CoreRule for TocScanner {

    fn run(root: &mut Node, _: &MarkdownIt) {
        let mut slugger = Slugger::default();
        let mut toc_builder = TocBuilder::new();
        root.walk(|node, _| {
            if node.is::<ATXHeading>() {
                let name = node.collect_text();
                toc_builder.push(node.cast::<ATXHeading>().unwrap().level, name.clone(), slugger.slug(&name));
            } 
        });
        let toc = toc_builder.into_toc();
        root.walk_mut(|node, _| {
            if node.is::<TocMarker>(){
                node.replace(toc.clone());
            }
        })
            
    }
}

#[derive(Debug)]
struct TocMarker;

impl NodeValue for TocMarker {
    fn render(&self, _node: &Node, _fmt: &mut dyn Renderer) {
        
    }

}

struct TocMarkerScanner;

impl InlineRule for TocMarkerScanner {
    const MARKER: char = '[';

    fn run(state:&mut InlineState) -> Option<(Node, usize)> {
        let input = &state.src[state.pos..state.pos_max]; 
        println!("{}", input);
        if input.contains("[toc!]") {
            println!("Marked TOC");
            return Some((Node::new(TocMarker), 6));
        }
        None
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
