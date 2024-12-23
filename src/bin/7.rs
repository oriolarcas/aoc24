use std::io::BufRead;

use anyhow::{bail, Context, Result};
use clap::Parser;

#[derive(Parser)]
struct Args {
    file: String,
}

#[derive(Debug)]
struct Equation {
    total: u64,
    operands: Vec<u64>,
}

fn parse_input(file_name: &str) -> Result<Vec<Equation>> {
    let file = std::fs::File::open(file_name);
    let reader = std::io::BufReader::new(file?);

    let mut lines = reader.lines();

    let mut equations = Vec::new();

    while let Some(line) = lines.next() {
        let line = line?;

        let mut equation = line.chars();

        let mut total: u64 = 0;

        while let Some(c) = equation.next() {
            if c.is_numeric() {
                total = total * 10 + u64::from(c.to_digit(10).context("invalid digit")?);
            } else if c == ':' {
                if equation.next().context("unexpected EOF")? != ' ' {
                    anyhow::bail!("unexpected: {}", c);
                }
                break;
            } else {
                anyhow::bail!("unexpected: {}", c);
            }
        }

        let mut operands = Vec::new();
        let mut operand = None;

        for c in equation {
            if c.is_numeric() {
                operand = Some(
                    operand.unwrap_or_default() * 10
                        + u64::from(c.to_digit(10).context("invalid digit")?),
                );
            } else if c == ' ' {
                operands.push(operand.context("unexpected space")?);
                operand = None;
            } else {
                anyhow::bail!("unexpected: {}", c);
            }
        }

        operands.push(operand.context("unexpected EOF")?);

        equations.push(Equation { total, operands });
    }

    Ok(equations)
}

fn concat(a: u64, b: u64) -> u64 {
    // print!("{a} || {b} = ");

    let mut a = a;
    let mut new_b = b;

    while new_b > 0 {
        a *= 10;
        new_b /= 10;
    }

    // println!("{}", a + b);

    a + b
}

fn is_valid_operands(expected: u64, partial: u64, operands: &[u64]) -> bool {
    if operands.is_empty() {
        return expected == partial;
    }

    let operand = operands[0];
    let rest = &operands[1..];

    is_valid_operands(expected, partial + operand, rest)
        || is_valid_operands(expected, partial * operand, rest)
}

fn is_valid_equation(equation: &Equation) -> bool {
    let operand = equation.operands[0];
    let rest = &equation.operands[1..];

    is_valid_operands(equation.total, operand, rest)
}

fn is_valid_operands_with_concat(expected: u64, partial: u64, operands: &[u64]) -> bool {
    if operands.is_empty() {
        return expected == partial;
    }

    let operand = operands[0];
    let rest = &operands[1..];

    is_valid_operands_with_concat(expected, partial + operand, rest)
        || is_valid_operands_with_concat(expected, partial * operand, rest)
        || is_valid_operands_with_concat(expected, concat(partial, operand), rest)
}

fn is_valid_equation_with_concat(equation: &Equation) -> bool {
    let operand = equation.operands[0];
    let rest = &equation.operands[1..];

    is_valid_operands_with_concat(equation.total, operand, rest)
}

fn fix_equations(file_name: &str) -> Result<(u64, u64)> {
    let mut valid_equations_sum = 0;
    let mut valid_equations_with_concat_sum = 0;

    let equations = parse_input(file_name)?;

    for equation in equations {
        if equation.operands.is_empty() {
            bail!("empty operands");
        }

        if is_valid_equation(&equation) {
            valid_equations_sum += equation.total;
        }

        if is_valid_equation_with_concat(&equation) {
            valid_equations_with_concat_sum += equation.total;
        }
    }

    Ok((valid_equations_sum, valid_equations_with_concat_sum))
}

fn main() -> Result<()> {
    let args = Args::parse();

    let (valid_equations_sum, valid_equations_with_concat_sum) = fix_equations(&args.file)?;

    println!("Valid equations sum: {valid_equations_sum}");
    println!("Valid equations with concat sum: {valid_equations_with_concat_sum}");

    Ok(())
}
