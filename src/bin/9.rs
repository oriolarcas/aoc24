use std::io::Read;

#[cfg(debug_assertions)]
use std::fmt::Display;

use anyhow::{bail, Result};
use clap::Parser;

#[derive(Parser)]
struct Args {
    file: String,
}

#[derive(Clone, PartialEq)]
enum Block {
    Some { id: usize },
    None,
}

#[derive(Clone)]
struct Disk {
    blocks: Vec<Block>,
    size: usize,
}

#[cfg(debug_assertions)]
impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Block::Some { id } => write!(f, "({id})")?,
            Block::None => write!(f, ".")?,
        };

        Ok(())
    }
}

#[cfg(debug_assertions)]
impl Display for Disk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for block in &self.blocks {
            write!(f, "{}", block)?;
        }
        write!(f, "]")?;

        Ok(())
    }
}

impl Disk {
    fn from_file(file_name: &str) -> Result<Self> {
        let file = std::fs::File::open(file_name);
        let reader = std::io::BufReader::new(file?);

        let mut blocks = Vec::new();

        // Let's assume 1 byte per digit
        let mut bytes = reader.bytes();
        let mut is_free_space = false;
        let mut file_id = 0;
        let mut size = 0;

        loop {
            let Some(digit) = bytes.next() else {
                break;
            };

            let digit = digit?;

            if digit == b'\n' {
                if !bytes.next().is_none() {
                    bail!("invalid file format");
                }
                break;
            }

            let digit = match digit {
                b'0'..=b'9' => digit - b'0',
                _ => bail!("invalid digit: {}", digit),
            };

            if !is_free_space && digit == 0 {
                bail!("invalid file size: 0");
            }

            if !(is_free_space && digit == 0) {
                for _ in 0..digit {
                    blocks.push(if !is_free_space {
                        Block::Some { id: file_id }
                    } else {
                        Block::None
                    });
                }

                if !is_free_space {
                    size += usize::from(digit);
                }
            }

            if !is_free_space {
                file_id += 1;
            }

            is_free_space = !is_free_space;
        }

        Ok(Disk { blocks, size })
    }

    fn defragment(&mut self) -> Result<usize> {
        let mut checksum = 0;

        let mut head_index = 0;
        let mut tail_index = self.blocks.len() - 1;

        while head_index < self.size {
            let id = match self.blocks[head_index] {
                Block::Some { id } => id,
                Block::None => {
                    while self.blocks[tail_index] == Block::None {
                        tail_index -= 1;
                    }

                    self.blocks.swap(head_index, tail_index);
                    match self.blocks[head_index] {
                        Block::Some { id } => id,
                        Block::None => bail!("invalid block"),
                    }
                }
            };

            checksum += head_index * id;
            head_index += 1;
        }

        Ok(checksum)
    }
}

fn checksum(file_name: &str) -> Result<(usize, usize)> {
    let mut disk = Disk::from_file(file_name)?;

    #[cfg(debug_assertions)]
    println!("{}", disk);

    let checksum = disk.defragment()?;

    #[cfg(debug_assertions)]
    println!("{}", disk);

    Ok((checksum, 0))
}

fn main() -> Result<()> {
    let args = Args::parse();

    let (checksum, _) = checksum(&args.file)?;

    println!("Checksum: {checksum}");

    Ok(())
}
