use std::io::BufRead as _;

use anyhow::Result;
use clap::Parser;

// Clap struct with a single positional argument: a path to a file
#[derive(Parser)]
struct Args {
    file: String,
}

fn stonum(first: Option<char>, chars: &mut std::str::Chars) -> u64 {
    let mut value: u64 = first.map(|c| c.to_digit(10).unwrap() as u64).unwrap_or(0);
    while let Some(c) = chars.next() {
        if c.is_ascii_whitespace() {
            break;
        }
        value = value * 10 + c.to_digit(10).unwrap() as u64;
    }
    value
}

fn skip_whitespace(chars: &mut std::str::Chars) -> Option<char> {
    while let Some(c) = chars.next() {
        if !c.is_ascii_whitespace() {
            return Some(c);
        }
    }
    None
}

fn file_list_distance(file_name: &str) -> Result<u64> {
    let file = std::fs::File::open(file_name)?;
    let reader = std::io::BufReader::new(file);

    let mut list_a = Vec::new();
    let mut list_b = Vec::new();
    for line in reader.lines() {
        let line = line?;

        let mut chars = line.chars();

        list_a.push(stonum(chars.next(), &mut chars));
        list_b.push(stonum(skip_whitespace(&mut chars), &mut chars));
    }

    list_a.sort();
    list_b.sort();

    let mut distance = 0;
    for (a, b) in list_a.iter().zip(list_b.iter()) {
        distance += a.abs_diff(*b);
    }

    Ok(distance)
}

fn file_list_similarity(file_name: &str) -> Result<u64> {
    let file = std::fs::File::open(file_name)?;
    let reader = std::io::BufReader::new(file);

    let mut list_a = Vec::new();
    let mut list_b = Vec::new();
    for line in reader.lines() {
        let line = line?;

        let mut chars = line.chars();

        list_a.push(stonum(chars.next(), &mut chars));
        list_b.push(stonum(skip_whitespace(&mut chars), &mut chars));
    }

    list_a.sort();
    list_b.sort();

    let mut similarity = 0;
    for a in list_a {
        let mut occurrences = 0;
        if let Ok(index) = list_b.binary_search(&a) {
            occurrences += 1;
            for left_index in (0..index).rev() {
                if list_b[left_index] == a {
                    occurrences += 1;
                } else {
                    break;
                }
            }
            for right_index in index + 1..list_b.len() {
                if list_b[right_index] == a {
                    occurrences += 1;
                } else {
                    break;
                }
            }
        }

        similarity += a * occurrences;
    }

    Ok(similarity)
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Distance: {}", file_list_distance(&args.file)?);
    println!("Similarity: {}", file_list_similarity(&args.file)?);

    Ok(())
}
