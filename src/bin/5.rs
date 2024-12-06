use std::{collections::HashSet, io::BufRead};

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
            if self.graph.contains_edge(*page_after, page_before) {
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

    // Based on https://en.wikipedia.org/wiki/Topological_sorting#Depth-first_search
    fn topological_sort(&self, nodes: &[u64]) -> Vec<u64> {
        let mut sorted = Vec::new();
        let visitable = nodes.iter().cloned().collect::<HashSet<_>>();
        let mut visited = HashSet::new();

        for node in nodes {
            if !visited.contains(node) {
                self.visit(*node, &visitable, &mut visited, &mut sorted);
            }
        }

        sorted.reverse();
        sorted
    }

    fn visit(
        &self,
        node: u64,
        nodes: &HashSet<u64>,
        visited: &mut HashSet<u64>,
        sorted: &mut Vec<u64>,
    ) {
        visited.insert(node);

        for neighbor in self.graph.neighbors(node) {
            if !nodes.contains(&neighbor) {
                // Skip nodes that are not in the original list
                continue;
            }

            if !visited.contains(&neighbor) {
                self.visit(neighbor, nodes, visited, sorted);
            }
        }

        sorted.push(node);
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
            let pages = ordering_rules.topological_sort(&pages);

            // println!(
            //     "+ Reordered: {}",
            //     pages
            //         .iter()
            //         .map(|page| page.to_string())
            //         .collect::<Vec<_>>()
            //         .join(", ")
            // );

            assert!(
                ordering_rules.is_valid(&pages),
                "Reordered pages {:?} are not valid",
                pages
            );

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
