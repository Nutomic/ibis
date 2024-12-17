use github_slugger::Slugger;
use leptos::{component, prelude::*, IntoView};

#[derive(Default)]
pub struct Toc {
    entries: Vec<TocEntry>,
}

impl Toc {
    fn count_entries_with_level(&self, level: u8) -> usize {
        self.entries.iter().filter(|e| e.level == level).count()
    }
}

struct TocEntry {
    level: u8,
    sec_number: String,
    name: String,
    id: String,
    children: Toc,
}

#[derive(Default)]
pub(crate) struct TocBuilder {
    top_level: Toc,
    chain: Vec<TocEntry>,
}

impl TocBuilder {
    pub(crate) fn new() -> TocBuilder {
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

    pub(crate) fn push(&mut self, level: u8, name: String, id: String) -> &str {
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

pub fn generate_table_of_contents(text: &str) -> Toc {
    let mut slugger = Slugger::default();
    let mut toc_builder = TocBuilder::new();
    text.lines()
        .filter(|x| {
            if !x.starts_with("#") {
                return false;
            }
            x.chars()
                .skip_while(|char| *char == '#')
                .collect::<String>()
                .starts_with(" ")
        })
        .for_each(|x| {
            let mut level: u8 = 0;
            let line = x
                .chars()
                .skip_while(|x| {
                    level += 1;
                    *x == '#' || *x == ' '
                })
                .collect::<String>();
            toc_builder.push(level - 2, line.to_string(), slugger.slug(&line));
        });
    toc_builder.into_toc()
}

#[component]
fn PrintInner(toc: Toc) -> impl IntoView {
    view! {
        <ul>
            {toc
                .entries
                .into_iter()
                .map(|entry| {
                    view! {
                        <li>
                            <a href="#".to_owned()
                                + { &entry.id }>{entry.sec_number}" "{entry.name}</a>
                            <PrintInner toc=entry.children />
                        </li>
                    }
                })
                .collect::<Vec<_>>()}
        </ul>
    }
    .into_any()
}

#[component]
pub fn Toc(text: String) -> impl IntoView {
    let toc = generate_table_of_contents(&text);
    if !toc.entries.is_empty() {
        view! {
            <div class="float-right mr-20 w-80 menu h-fit rounded-box">
                <li class="menu-title">Table of Contents</li>
                <PrintInner toc=toc />
            </div>
        }
        .into_any()
    } else {
        ().into_any()
    }
}
