use github_slugger::Slugger;
use markdown_it::{
    MarkdownIt,
    Node,
    NodeValue,
    Renderer,
    parser::{
        core::CoreRule,
        inline::{InlineRule, InlineState},
    },
    plugins::cmark::block::{heading::ATXHeading, lheading::SetextHeader},
};

#[derive(Debug, Default, Clone)]
pub struct Toc {
    entries: Vec<TocEntry>,
}

impl Toc {
    fn count_entries_with_level(&self, level: u8) -> usize {
        self.entries.iter().filter(|e| e.level == level).count()
    }

    fn print_entry(&self, fmt: &mut dyn Renderer) {
        fmt.open("ul", &[]);
        self.entries.iter().for_each(|entry| {
            fmt.open("li", &[]);
            fmt.open("a", &[("href", "#".to_owned() + &entry.id)]);

            fmt.text(&format!("{} {}", entry.sec_number, entry.name));

            fmt.close("a");

            entry.children.print_entry(fmt);

            fmt.close("li");
        });

        fmt.close("ul");
    }
}

impl NodeValue for Toc {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        let mut attrs = node.attrs.clone();

        attrs.push(("class", "not-prose mr-20 w-80 menu rounded-box".into()));

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

    fn push(&mut self, level: u8, name: String, id: String) -> String {
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

        self.chain
            .last()
            .map(|l| l.sec_number.clone())
            .unwrap_or_default()
    }
}

pub struct TocScanner;

impl CoreRule for TocScanner {
    fn run(root: &mut Node, _: &MarkdownIt) {
        let mut slugger = Slugger::default();
        let mut toc_builder = TocBuilder::default();
        root.walk(|node, _| {
            if node.is::<ATXHeading>() || node.is::<SetextHeader>() {
                let name = node.collect_text();
                toc_builder.push(
                    node.cast::<ATXHeading>()
                        .map(|h| h.level)
                        .unwrap_or_else(|| {
                            node.cast::<SetextHeader>()
                                .map(|h| h.level)
                                .unwrap_or_default()
                        }),
                    name.clone(),
                    slugger.slug(&name),
                );
            }
        });
        let toc = toc_builder.into_toc();
        root.walk_mut(|node, _| {
            if node.is::<TocMarker>() {
                node.replace(toc.clone());
            }
        })
    }
}

#[derive(Debug)]
struct TocMarker;

impl NodeValue for TocMarker {
    fn render(&self, _node: &Node, _fmt: &mut dyn Renderer) {}
}

pub struct TocMarkerScanner;

impl InlineRule for TocMarkerScanner {
    const MARKER: char = '[';

    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        let input = &state.src[state.pos..state.pos_max];
        if input.contains("[!toc]") {
            return Some((Node::new(TocMarker), 6));
        }
        None
    }
}
