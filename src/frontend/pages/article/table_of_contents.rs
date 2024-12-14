
#[derive(PartialEq)]
struct Toc {
    entries: Vec<TocEntry>,
}

impl Toc {
    fn count_entries_with_level(&self, level: u32) -> usize {
        self.entries.iter().filter(|e| e.level == level).count()
    }
}

#[derive(PartialEq)]
struct TocEntry {
    level: u32,
    sec_number: String,
    name: String,
    id: String,
    children: Toc,
}

#[derive(PartialEq)]
pub(crate) struct TocBuilder {
    top_level: Toc,
    chain: Vec<TocEntry>,
}

impl TocBuilder {
    pub(crate) fn new() -> TocBuilder {
        TocBuilder { top_level: Toc { entries: Vec::new() }, chain: Vec::new() }
    }

    pub fn into_toc(mut self) -> Toc {
        self.fold_until(0);
        self.top_level
    }

    fn fold_until(&mut self, level: u32) {
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

    pub(crate) fn push(&mut self, level: u32, name: String, id: String) -> &str {
        assert!(level >= 1);

        self.fold_until(level);

        let mut sec_number;
        {
            let (toc_level, toc) = match self.chain.last() {
                None => {
                    sec_number = String::new();
                    (0, &self.top_level)
                }
                Some(entry) => {
                    sec_number = entry.sec_number.clone();
                    sec_number.push('.');
                    (entry.level, &entry.children)
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
            children: Toc { entries: Vec::new() },
        });

        let just_inserted = self.chain.last_mut().unwrap();
        &just_inserted.sec_number
    }
}

impl Toc {
    fn print_inner(&self, v: &mut String) {
        use std::fmt::Write as _;

        v.push_str("<ul>");
        for entry in &self.entries {
            let _ = write!(
                v,
                "\n<li><a href=\"#{id}\">{num} {name}</a>",
                id = entry.id,
                num = entry.sec_number,
                name = entry.name
            );
            entry.children.print_inner(&mut *v);
            v.push_str("</li>");
        }
        v.push_str("</ul>");
    }
    pub(crate) fn print(&self) -> String {
        let mut v = String::new();
        self.print_inner(&mut v);
        v
    }
}

pub fn generate_table_of_contents(text: &str) -> String {
    let mut toc_builder = TocBuilder::new();
    text.lines()
        .filter(
            |x|{
                if !x.starts_with("#") {
                    return false;
                }
                x.chars()
                .skip_while(|char| *char == '#' )
                .collect::<String>()
                .starts_with(" ")
            }
        )
        .for_each(
        |x| {
            println!("{}", x);
            let mut level: u32 = 0;
            let line = x.chars()
                .skip_while(
                    |x| {
                        level += 1;
                        *x == '#' || *x == ' '
                    }
                ).collect::<String>();
            toc_builder.push(
                    level - 2,
                    line.clone().to_string(),
                    to_kebab_case(line)
                );

        }
        );
    let toc = toc_builder.into_toc();
    println!("{}", toc.print());
    toc.print()
}

pub fn to_kebab_case(line: String) -> String {
    return line.to_lowercase()
        .chars()
        .filter(|x| x.is_alphabetic() || *x == ' ')
        .collect::<String>()
        .replace(" ", "-");
}
