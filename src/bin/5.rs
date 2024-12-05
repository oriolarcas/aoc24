use std::{
    collections::{HashMap, HashSet},
    io::BufRead,
};

use anyhow::Result;
use clap::Parser;
use petgraph::prelude::DiGraphMap;

#[derive(Parser)]
struct Args {
    file: String,
}

struct OrderingRules {
    graph: DiGraphMap<u64, u64>,
}

impl OrderingRules {
    fn new() -> Self {
        Self {
            graph: DiGraphMap::new(),
        }
    }

    fn add_rule(&mut self, before: u64, after: u64) {
        self.graph.add_edge(before, after, 1);
    }

    fn is_valid_partial(&self, page_before: u64, next_pages: &[u64]) -> bool {
        let mut valid = true;
        for page_after in next_pages {
            if self.graph.contains_edge(page_before, *page_after) {
                valid = false;
            }
        }

        valid
    }

    fn is_valid(&self, pages: &[u64]) -> bool {
        let mut valid = true;
        for (index, page_before) in pages.iter().take(pages.len() - 1).enumerate() {
            let next_pages = &pages[index + 1..];

            if !self.is_valid_partial(*page_before, next_pages) {
                valid = false;
            }
        }

        valid
    }
}

fn reorder_update(pages: &mut Vec<u64>, order_rules: &HashMap<u64, HashSet<u64>>) {
    // This is a horrible bubble sort algorithm. It is not efficient, but it is simple and it works.
    // The ordering rules could be used to build a topological sort, but who has time for that?
    loop {
        let mut swapped = false;
        for index in 0..pages.len() {
            let mut index_before = index;
            let page = pages[index];

            for index_after in index + 1..pages.len() {
                let page_after = pages[index_after];

                if let Some(cannot_be_before) = order_rules.get(&page_after) {
                    if cannot_be_before.contains(&page) {
                        pages.swap(index_before, index_after);
                        index_before = index_after;
                        swapped = true;
                    }
                }
            }
        }

        if !swapped {
            break;
        }
    }
}

fn verify_updates(file_name: &str) -> Result<(u64, u64)> {
    let file = std::fs::File::open(file_name);
    let reader = std::io::BufReader::new(file?);

    let mut valid_middle_page_sum = 0;
    let mut reordered_middle_page_sum = 0;

    let mut ordering_rules = OrderingRules::new();

    let mut lines = reader.lines();
    while let Some(line) = lines.next() {
        let line = line?;

        if line.is_empty() {
            break;
        }

        let mut pages = line.split('|');
        let before = pages.next().unwrap().trim().parse::<u64>()?;
        let after = pages.next().unwrap().trim().parse::<u64>()?;

        ordering_rules.add_rule(before, after);
    }

    while let Some(line) = lines.next() {
        let line = line?;

        // println!("Update: {}", line);

        let pages = line
            .split(',')
            .map(|page| page.trim().parse::<u64>())
            .collect::<Result<Vec<_>, _>>()?;

        if ordering_rules.is_valid(&pages) {
            // println!("+ Is valid");
            let middle_page = pages[pages.len() / 2];
            valid_middle_page_sum += middle_page;
        } else {
            let mut pages = pages;
            // reorder_update(&mut pages, &order_rules);

            // println!(
            //     "+ Reordered: {}",
            //     pages
            //         .iter()
            //         .map(|page| page.to_string())
            //         .collect::<Vec<_>>()
            //         .join(", ")
            // );

            // assert!(
            //     ordering_rules.is_valid(&pages),
            //     "Reordered pages {:?} are not valid",
            //     pages
            // );

            let middle_page = pages[pages.len() / 2];
            reordered_middle_page_sum += middle_page;
        }
    }

    Ok((valid_middle_page_sum, reordered_middle_page_sum))
}

fn main() -> Result<()> {
    let args = Args::parse();

    let (valid_middle_page_sum, reordered_middle_page_sum) = verify_updates(&args.file)?;

    println!("Valid updates: {valid_middle_page_sum}");
    println!("Reordered updates: {reordered_middle_page_sum}");

    Ok(())
}
