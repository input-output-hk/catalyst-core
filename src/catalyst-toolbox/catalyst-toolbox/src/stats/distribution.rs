use core::ops::Range;
use std::collections::HashMap;

fn levels(threshold: u64) -> Vec<Range<u64>> {
    vec![
        (0..450),
        (450..threshold),
        (threshold..1_000),
        (1_000..2_000),
        (2_000..5_000),
        (5_000..10_000),
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
        Self::new_with_levels(levels(threshold))
    }

    pub fn new_with_levels(levels: Vec<Range<u64>>) -> Self {
        Self {
            content: levels
                .into_iter()
                .map(|range| (range, Default::default()))
                .collect(),
        }
    }

    pub fn add_with_weight(&mut self, value: u64, weight: u32) {
        for (range, record) in self.content.iter_mut() {
            if range.contains(&value) {
                record.count += weight;
                record.total += value;
                return;
            }
        }
    }

    pub fn add(&mut self, value: u64) {
        self.add_with_weight(value, 1);
    }

    pub fn print_count_per_level(&self) {
        let mut keys = self.content.keys().cloned().collect::<Vec<Range<u64>>>();
        keys.sort_by_key(|x| x.start);

        for key in keys {
            let start = format_big_number(key.start);
            let end = format_big_number(key.end);
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
            let start = format_big_number(key.start);
            let end = format_big_number(key.end);
            println!(
                "{} .. {} -> {} ",
                start,
                end,
                format_big_number(self.content[&key].total)
            );
        }
        println!(
            "Total -> {} ",
            self.content.values().map(|x| x.total).sum::<u64>()
        );
    }
}

fn format_big_number(n: u64) -> String {
    if n == 0 {
        n.to_string()
    } else if n % 1_000_000_000 == 0 {
        format!("{} MLD", n / 1_000_000)
    } else if n % 1_00000 == 0 {
        format!("{} M", n / 1_000_000)
    } else if n % 1_000 == 0 {
        format!("{} k", n / 1_000)
    } else {
        n.to_string()
    }
}
