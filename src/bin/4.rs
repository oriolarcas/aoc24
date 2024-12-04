use std::io::BufRead;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
struct Args {
    file: String,
}

fn find_all(s: &[char], pattern: &[char]) -> u64 {
    let mut count = 0;

    for window in s.windows(pattern.len()) {
        if window == pattern {
            count += 1;
        }
    }

    let rev_pattern = pattern.iter().rev().copied().collect::<Vec<_>>();

    for window in s.windows(rev_pattern.len()) {
        if window == rev_pattern {
            count += 1;
        }
    }

    count
}

fn xmas_count_1d(file_name: &str) -> Result<u64> {
    const PATTERN: &[char] = &['X', 'M', 'A', 'S'];

    let file = std::fs::File::open(file_name)?;
    let reader = std::io::BufReader::new(file);

    let mut count = 0;

    let puzzle = reader
        .lines()
        .map(|s| s.map(|s| s.chars().collect::<Vec<_>>()))
        .collect::<Result<Vec<_>, _>>()?;

    let width = puzzle.first().unwrap().len();
    let height = puzzle.len();

    // Search for "XMAS" in the rows
    count += puzzle
        .iter()
        .map(|row| find_all(&row, PATTERN))
        .sum::<u64>();

    // Search for "XMAS" in the columns
    for index in 0..width {
        let column = puzzle
            .iter()
            .map(|row| *row.get(index).unwrap())
            .collect::<Vec<_>>();
        count += find_all(&column, PATTERN);
    }

    // Search for "XMAS" in the diagonals starting from the leftmost column
    for i in 0..height {
        let mut diagonal1 = Vec::new();
        let mut diagonal2 = Vec::new();

        for j in 0..height.max(width) {
            if i + j < height && j < width {
                diagonal1.push(*puzzle.get(i + j).unwrap().get(j).unwrap());
                diagonal2.push(*puzzle.get(i + j).unwrap().get(width - j - 1).unwrap());
            }
        }

        count += find_all(&diagonal1, PATTERN);
        count += find_all(&diagonal2, PATTERN);
    }

    // Search for "XMAS" in the diagonals starting from the top row
    for i in 1..width {
        let mut diagonal1 = Vec::new();
        let mut diagonal2 = Vec::new();

        for j in 0..height.max(width) {
            if j < height && i + j < width {
                diagonal1.push(*puzzle.get(j).unwrap().get(i + j).unwrap());
                diagonal2.push(*puzzle.get(j).unwrap().get(width - i - j - 1).unwrap());
            }
        }

        count += find_all(&diagonal1, PATTERN);
        count += find_all(&diagonal2, PATTERN);
    }

    Ok(count)
}

fn xmas_count_2d(file_name: &str) -> Result<u64> {
    const PATTERN: &[char] = &['M', 'A', 'S'];

    let file = std::fs::File::open(file_name)?;
    let reader = std::io::BufReader::new(file);

    let mut count = 0;

    let puzzle = reader
        .lines()
        .map(|s| s.map(|s| s.chars().collect::<Vec<_>>()))
        .collect::<Result<Vec<_>, _>>()?;

    let width = puzzle.first().unwrap().len();
    let height = puzzle.len();

    let rev_pattern = PATTERN.iter().rev().copied().collect::<Vec<_>>();

    for i in 0..height {
        if i + PATTERN.len() > height {
            break;
        }

        for j in 0..width {
            if j + PATTERN.len() > width {
                break;
            }

            let diagonal1 = (0..PATTERN.len())
                .map(|k| *puzzle.get(i + k).unwrap().get(j + k).unwrap())
                .collect::<Vec<_>>();
            let diagonal2 = (0..PATTERN.len())
                .map(|k| {
                    *puzzle
                        .get(i + k)
                        .unwrap()
                        .get(j + PATTERN.len() - k - 1)
                        .unwrap()
                })
                .collect::<Vec<_>>();

            if (diagonal1 == PATTERN || diagonal1 == rev_pattern)
                && (diagonal2 == PATTERN || diagonal2 == rev_pattern)
            {
                count += 1;
            }
        }
    }

    Ok(count)
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("XMAS count: {}", xmas_count_1d(&args.file)?);
    println!("X-MAS count: {}", xmas_count_2d(&args.file)?);

    Ok(())
}
