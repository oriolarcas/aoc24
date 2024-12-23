use std::collections::{HashMap, HashSet};
use std::io::BufRead;

#[cfg(debug_assertions)]
use std::fmt::Display;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
struct Args {
    file: String,
}

#[derive(Debug)]
struct Vector {
    x: i64,
    y: i64,
}

#[derive(Clone, Eq, PartialEq, Hash)]
struct Location {
    x: i64,
    y: i64,
}

impl Location {
    fn add(&self, vector: &Vector) -> Self {
        Self {
            x: self.x + vector.x,
            y: self.y + vector.y,
        }
    }

    fn sub(&self, location: &Location) -> Vector {
        Vector {
            x: self.x - location.x,
            y: self.y - location.y,
        }
    }
}

#[derive(Clone)]
struct Map {
    antennas: HashMap<char, Vec<Location>>,
    width: i64,
    height: i64,
    #[cfg(debug_assertions)]
    antinodes: Vec<Location>,
}

#[cfg(debug_assertions)]
impl Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = vec![vec!['.'; self.width as usize]; self.height as usize];

        for (freq, locations) in self.antennas.iter() {
            for location in locations {
                map[location.y as usize][location.x as usize] = *freq;
            }
        }

        for antinode in self.antinodes.iter() {
            map[antinode.y as usize][antinode.x as usize] = '#';
        }

        for row in map {
            writeln!(f, "{}", row.iter().collect::<String>())?;
        }

        Ok(())
    }
}

impl Map {
    fn from_file(file_name: &str) -> Result<Map> {
        let file = std::fs::File::open(file_name);
        let reader = std::io::BufReader::new(file?);

        let mut antennas = HashMap::new();
        let mut width = 0;
        let mut height = 0;

        for (row, line) in reader.lines().enumerate() {
            let line = line?;

            if line.is_empty() {
                continue;
            }

            if width != 0 {
                if width != line.len() as i64 {
                    return Err(anyhow::anyhow!("invalid map"));
                }
            }

            height += 1;
            width = line.len() as i64;

            line.chars()
                .enumerate()
                .filter(|(_, c)| *c != '.')
                .try_for_each(|(col, c)| {
                    antennas.entry(c).or_insert_with(Vec::new).push(Location {
                        x: i64::try_from(col)?,
                        y: i64::try_from(row)?,
                    });
                    Result::<(), anyhow::Error>::Ok(())
                })?;
        }

        Ok(Map {
            antennas,
            width,
            height,
            #[cfg(debug_assertions)]
            antinodes: Vec::new(),
        })
    }

    fn is_valid_antinode(&self, location: &Location) -> bool {
        location.x >= 0 && location.x < self.width && location.y >= 0 && location.y < self.height
    }

    fn antinodes(&self, antenna_a: &Location, antenna_b: &Location) -> Vec<Location> {
        let vector = antenna_a.sub(antenna_b);

        let mut antinodes = Vec::new();

        let mut antenna_a = antenna_a.clone();

        loop {
            antenna_a = antenna_a.add(&vector);

            if !self.is_valid_antinode(&antenna_a) {
                break;
            }

            antinodes.push(antenna_a.clone());
        }

        antinodes
    }

    #[cfg(debug_assertions)]
    fn antinodes_map(&self, antenna_a: &Location, antenna_b: &Location) -> Result<Self> {
        let mut antinodes = self.antinodes(antenna_a, antenna_b);
        antinodes.extend(self.antinodes(antenna_b, antenna_a));

        let freq = self
            .antennas
            .iter()
            .find(|(_, antennas)| antennas.contains(antenna_a) && antennas.contains(antenna_b))
            .map(|(freq, _)| *freq)
            .expect("invalid antennas");

        Ok(Map {
            antennas: HashMap::from([(freq, vec![antenna_a.clone(), antenna_b.clone()])]),
            antinodes,
            ..self.clone()
        })
    }
}

struct Pairs<'a, T> {
    elements: &'a [T],
    index: usize,
    subindex: usize,
}

impl<'a, T> Pairs<'a, T> {
    fn new(elements: &'a [T]) -> Self {
        Self {
            elements,
            index: 0,
            subindex: 1,
        }
    }
}

impl<'a, T> Iterator for Pairs<'a, T> {
    type Item = (&'a T, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        // If only one item is left
        if self.index + 1 >= self.elements.len() {
            return None;
        }

        let pair = (&self.elements[self.index], &self.elements[self.subindex]);

        self.subindex += 1;

        if self.subindex >= self.elements.len() {
            self.index += 1;
            self.subindex = self.index + 1;
        }

        Some(pair)
    }
}

fn detect_antinodes(file_name: &str) -> Result<(usize, usize)> {
    let map = Map::from_file(file_name)?;
    let mut antinodes_simple = HashSet::<Location>::new();
    let mut antinodes_extended =
        HashSet::<Location>::from_iter(map.antennas.values().flatten().cloned());

    #[cfg(debug_assertions)]
    println!("{}", map);

    for (_freq, antennas) in map.antennas.iter() {
        #[cfg(debug_assertions)]
        println!("Frequency {}: {} antennas", _freq, antennas.len());

        for (antenna_a, antenna_b) in Pairs::new(antennas) {
            let antinodes_ab = map.antinodes(antenna_a, antenna_b);
            let antinodes_ba = map.antinodes(antenna_b, antenna_a);

            antinodes_simple.extend(antinodes_ab.first().cloned());
            antinodes_simple.extend(antinodes_ba.first().cloned());

            antinodes_extended.extend(antinodes_ab);
            antinodes_extended.extend(antinodes_ba);

            #[cfg(debug_assertions)]
            println!("{}", map.antinodes_map(antenna_a, antenna_b)?);
        }
    }

    Ok((antinodes_simple.len(), antinodes_extended.len()))
}

fn main() -> Result<()> {
    let args = Args::parse();

    let (antinodes, antinodes_extended) = detect_antinodes(&args.file)?;

    println!("Antinodes: {antinodes}");
    println!("Antinodes extended: {antinodes_extended}");

    Ok(())
}
