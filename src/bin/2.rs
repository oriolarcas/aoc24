use std::io::BufRead as _;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
struct Args {
    file: String,
}

fn report_is_safe(report: &str) -> bool {
    let mut prev: Option<u32> = None;
    let mut is_positive: Option<bool> = None;

    for level in report.split_whitespace() {
        let level = level.parse::<u32>().unwrap();
        if let Some(prev) = prev {
            let diff = i64::from(level) - i64::from(prev);
            if !(1..=3).contains(&diff.abs()) {
                return false;
            }
            let current_is_positive = match diff.signum() {
                1 => true,
                -1 => false,
                _ => return false,
            };
            if let Some(is_positive) = is_positive {
                if is_positive != current_is_positive {
                    return false;
                }
            }
            is_positive = Some(current_is_positive);
        }
        prev = Some(level);
    }

    true
}

fn is_safe(file_name: &str) -> Result<u64> {
    let file = std::fs::File::open(file_name);
    let reader = std::io::BufReader::new(file?);

    let mut safe_reports = 0;

    for line in reader.lines() {
        let line = line?;

        if report_is_safe(&line) {
            safe_reports += 1;
        }
    }

    Ok(safe_reports)
}

fn report_is_safe_recursive(
    prev: Option<u32>,
    is_positive: Option<bool>,
    dampened: bool,
    levels: &[u32],
) -> bool {
    if levels.is_empty() {
        return true;
    }

    let level = levels[0];
    if let Some(prev) = prev {
        // println!("  {} vs {}", prev, level);

        let diff = i64::from(level) - i64::from(prev);
        let check_levels = || {
            let current_is_positive = match diff.signum() {
                1 => true,
                -1 => false,
                _ => {
                    // println!("  {} == {}", prev, level);
                    return Err(())
                },
            };

            if !(1..=3).contains(&diff.abs()) {
                // println!("  {} is too far from {}", level, prev);
                return Err(());
            }
            if let Some(is_positive) = is_positive {
                if is_positive != current_is_positive {
                    // println!("  {} has the wrong sign", level);
                    return Err(());
                }
            }
            Ok(current_is_positive)
        };

        let skip_current = || {
            if dampened {
                return false;
            }
            // println!("  Skipping level {}", level);
            report_is_safe_recursive(Some(prev), is_positive, true, &levels[1..])
        };

        let Ok(current_is_positive) = check_levels() else {
            return skip_current();
        };

        if report_is_safe_recursive(
            Some(level),
            Some(current_is_positive),
            dampened,
            &levels[1..],
        ) {
            return true;
        }

        return skip_current();
    }

    if report_is_safe_recursive(Some(level), is_positive, dampened, &levels[1..]) {
        return true;
    }

    if !dampened {
        // Skip the current level and try again
        // println!("  Skipping head level {}", level);
        return report_is_safe_recursive(None, is_positive, true, &levels[1..]);
    }

    false
}

fn is_safe_with_dampening(file_name: &str) -> Result<u64> {
    let file = std::fs::File::open(file_name);
    let reader = std::io::BufReader::new(file?);

    let mut safe_reports = 0;

    for line in reader.lines() {
        let line = line?;

        // println!("{}", line);
        let levels: Vec<u32> = line
            .split_whitespace()
            .map(|level| level.parse().unwrap())
            .collect();

        if report_is_safe_recursive(None, None, false, &levels) {
            // println!("  Safe");
            safe_reports += 1;
        }
    }

    Ok(safe_reports)
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Safe reports: {}", is_safe(&args.file)?);
    println!(
        "Safe reports with error dampening: {}",
        is_safe_with_dampening(&args.file)?
    );

    Ok(())
}
