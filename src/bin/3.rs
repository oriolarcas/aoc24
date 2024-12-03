use std::io::Read;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
struct Args {
    file: String,
}

fn sanitized_mult(file_name: &str) -> Result<u64> {
    let file = std::fs::File::open(file_name)?;
    let mut reader = std::io::BufReader::new(file);

    let mut sum = 0;

    // Read the whole file
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer)?;

    let re = regex::Regex::new(r"mul\((\d+),(\d+)\)")?;

    for expr in re.captures_iter(&buffer) {
        let a: u64 = expr[1].parse()?;
        let b: u64 = expr[2].parse()?;

        sum += a * b;
    }

    Ok(sum)
}

fn sanitized_mult_with_conditions(file_name: &str) -> Result<u64> {
    let file = std::fs::File::open(file_name)?;
    let mut reader = std::io::BufReader::new(file);

    let mut sum = 0;

    // Read the whole file
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer)?;

    let re = regex::Regex::new(r"mul\((\d+),(\d+)\)|do\(\)|don't\(\)")?;

    let mut enabled = true;

    for expr in re.captures_iter(&buffer) {
        let expr_str = expr.get(0).unwrap().as_str();

        if expr_str == "do()" {
            enabled = true;
            continue;
        } else if expr_str == "don't()" {
            enabled = false;
            continue;
        } else if !enabled {
            continue;
        }

        let a: u64 = expr[1].parse()?;
        let b: u64 = expr[2].parse()?;

        sum += a * b;
    }

    Ok(sum)
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Sum: {}", sanitized_mult(&args.file)?);
    println!(
        "Sum with conditions: {}",
        sanitized_mult_with_conditions(&args.file)?
    );

    Ok(())
}
