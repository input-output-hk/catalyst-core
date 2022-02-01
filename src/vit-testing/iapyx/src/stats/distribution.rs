use core::ops::Range;
use regex::Regex;
use std::collections::HashMap;

fn levels(threshold: u64) -> Vec<Range<u64>> {
    vec![
        (0..threshold),
        (threshold..10_000),
        (10_000..20_000),
        (20_000..50_000),
        (50_000..100_000),
        (100_000..250_000),
        (250_000..500_000),
        (500_000..1_000_000),
        (1_000_000..5_000_000),
        (5_000_000..10_000_000),
        (10_000_000..25_000_000),
        (25_000_000..50_000_000),
        (50_000_000..32_000_000_000),
    ]
}

#[derive(Default)]
pub struct Record {
    pub count: u32,
    pub total: u64,
}

pub struct Stats {
    pub content: HashMap<Range<u64>, Record>,
}

impl Stats {
    pub fn new(threshold: u64) -> Self {
        Self {
            content: levels(threshold)
                .into_iter()
                .map(|range| (range, Default::default()))
                .collect(),
        }
    }

    pub fn add(&mut self, value: u64) {
        for (range, record) in self.content.iter_mut() {
            if range.contains(&value) {
                record.count += 1;
                record.total += value;
                return;
            }
        }
    }

    pub fn print_count_per_level(&self) {
        let mut keys = self.content.keys().cloned().collect::<Vec<Range<u64>>>();
        keys.sort_by_key(|x| x.start);

        for key in keys {
            let start = format_big_number(key.start.to_string());
            let end = format_big_number(key.end.to_string());
            println!("{} .. {} -> {} ", start, end, self.content[&key].count);
        }
        println!(
            "Total -> {} ",
            self.content.values().map(|x| x.count).sum::<u32>()
        );
    }

    pub fn print_ada_per_level(&self) {
        let mut keys = self.content.keys().cloned().collect::<Vec<Range<u64>>>();
        keys.sort_by_key(|x| x.start);

        for key in keys {
            let start = format_big_number(key.start.to_string());
            let end = format_big_number(key.end.to_string());
            println!(
                "{} .. {} -> {} ",
                start,
                end,
                format_big_number(self.content[&key].total.to_string())
            );
        }
        println!(
            "Total -> {} ",
            self.content.values().map(|x| x.total).sum::<u64>()
        );
    }
}

fn format_big_number<S: Into<String>>(number: S) -> String {
    #[allow(clippy::trivial_regex)]
    let mld = Regex::new(r"000000000$").unwrap();
    #[allow(clippy::trivial_regex)]
    let mln = Regex::new(r"000000$").unwrap();
    #[allow(clippy::trivial_regex)]
    let k = Regex::new(r"000$").unwrap();

    let mut output = mld.replace_all(&number.into(), " MLD").to_string();
    output = mln.replace_all(&output, " M").to_string();
    k.replace_all(&output, " k").to_string()
}
