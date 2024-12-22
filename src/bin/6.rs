use std::{fmt::Display, io::BufRead};

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
struct Args {
    file: String,
}

#[derive(Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn from_char(c: char) -> Result<Self> {
        match c {
            '^' => Ok(Self::Up),
            'v' => Ok(Self::Down),
            '<' => Ok(Self::Left),
            '>' => Ok(Self::Right),
            _ => Err(anyhow::anyhow!("Invalid direction: {c}")),
        }
    }

    fn rotate(&mut self) {
        *self = match self {
            Self::Up => Self::Right,
            Self::Right => Self::Down,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
        }
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Self::Up => '^',
            Self::Down => 'v',
            Self::Left => '<',
            Self::Right => '>',
        };

        write!(f, "{c}")
    }
}

#[derive(Clone)]
struct Guard {
    x: usize,
    y: usize,
    direction: Direction,
}

#[derive(Clone, Default)]
struct Visited {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

impl Visited {
    fn visit(&mut self, direction: &Direction) {
        match direction {
            Direction::Up => self.up = true,
            Direction::Down => self.down = true,
            Direction::Left => self.left = true,
            Direction::Right => self.right = true,
        }
    }

    fn is_visited(&self) -> bool {
        self.up || self.down || self.left || self.right
    }

    fn has_direction(&self, direction: &Direction) -> bool {
        match direction {
            Direction::Up => self.up,
            Direction::Down => self.down,
            Direction::Left => self.left,
            Direction::Right => self.right,
        }
    }
}

#[derive(Clone)]
struct Map {
    obstacles: Vec<Vec<bool>>,
    potential_obstacle: Option<(usize, usize)>,
    visited: Vec<Vec<Visited>>,
    guard: Guard,
}

enum Movement {
    Moved(usize, usize),
    HitObstacle,
    OutOfMap,
}

impl Map {
    fn from_file(file_name: &str) -> Result<Self> {
        let file = std::fs::File::open(file_name);
        let reader = std::io::BufReader::new(file?);

        let mut obstacles = Vec::new();
        let mut guard = Guard {
            x: 0,
            y: 0,
            direction: Direction::Up,
        };

        for (row, line) in reader.lines().enumerate() {
            let line = line?;

            if line.is_empty() {
                continue;
            }

            obstacles.push(
                line.chars()
                    .enumerate()
                    .map(|(column, c)| match c {
                        '.' => Ok(false),
                        '#' => Ok(true),
                        _ => {
                            guard = Guard {
                                x: column,
                                y: row,
                                direction: Direction::from_char(c)?,
                            };
                            Ok(false)
                        }
                    })
                    .collect::<Result<Vec<_>>>()?,
            );
        }

        let visited = vec![vec![Visited::default(); obstacles[0].len()]; obstacles.len()];

        Ok(Self {
            obstacles,
            visited,
            potential_obstacle: None,
            guard,
        })
    }

    fn can_move_from(&self, x: usize, y: usize, direction: Direction) -> Movement {
        match direction {
            Direction::Up => {
                if y == 0 {
                    Movement::OutOfMap
                } else if self.obstacles[y - 1][x] {
                    Movement::HitObstacle
                } else {
                    Movement::Moved(x, y - 1)
                }
            }
            Direction::Down => {
                if y == self.obstacles.len() - 1 {
                    Movement::OutOfMap
                } else if self.obstacles[y + 1][x] {
                    Movement::HitObstacle
                } else {
                    Movement::Moved(x, y + 1)
                }
            }
            Direction::Left => {
                if x == 0 {
                    Movement::OutOfMap
                } else if self.obstacles[y][x - 1] {
                    Movement::HitObstacle
                } else {
                    Movement::Moved(x - 1, y)
                }
            }
            Direction::Right => {
                if x == self.obstacles[0].len() - 1 {
                    Movement::OutOfMap
                } else if self.obstacles[y][x + 1] {
                    Movement::HitObstacle
                } else {
                    Movement::Moved(x + 1, y)
                }
            }
        }
    }

    fn patrol_has_loop(&mut self) -> bool {
        loop {
            match self.can_move_from(self.guard.x, self.guard.y, self.guard.direction) {
                Movement::Moved(x, y) => {
                    if self.visited[y][x].has_direction(&self.guard.direction) {
                        return true;
                    }

                    self.visited[y][x].visit(&self.guard.direction);

                    self.guard.x = x;
                    self.guard.y = y;
                }
                Movement::HitObstacle => {
                    self.guard.direction.rotate();
                    self.visited[self.guard.y][self.guard.x].visit(&self.guard.direction);
                }
                Movement::OutOfMap => {
                    return false;
                }
            }
        }
    }

    fn patrol(&mut self) -> (u64, u64) {
        self.visited[self.guard.y][self.guard.x].visit(&self.guard.direction);

        let mut visited_count = 1;
        let mut loop_obstacles_count = 0;
        let mut loop_obstacles = vec![vec![false; self.obstacles[0].len()]; self.obstacles.len()];
        let initial_x = self.guard.x;
        let initial_y = self.guard.y;

        loop {
            // println!("{}", self);
            // println!();

            match self.can_move_from(self.guard.x, self.guard.y, self.guard.direction) {
                Movement::Moved(x, y) => {
                    // let prev_x = self.guard.x;
                    // let prev_y = self.guard.y;

                    if !self.obstacles[y][x]
                        && !loop_obstacles[y][x]
                        && !(x == initial_x && y == initial_y)
                        && !self.visited[y][x].is_visited()
                    {
                        let mut simulated_map = self.clone();
                        simulated_map.obstacles[y][x] = true;

                        if simulated_map.patrol_has_loop() {
                            loop_obstacles[y][x] = true;
                            loop_obstacles_count += 1;

                            // let mut new_map = self.clone();
                            // new_map.potential_obstacle = Some((x, y));

                            // let mut rotated_direction = self.guard.direction;
                            // rotated_direction.rotate();

                            // println!("Potential loop detected if placing an obstacle at ({x}, {y}) and then going {} from ({prev_x}, {prev_y}):", rotated_direction);
                            // println!("{}", new_map);
                            // println!();
                        }
                    }

                    self.guard.x = x;
                    self.guard.y = y;

                    if !self.visited[y][x].is_visited() {
                        visited_count += 1;
                    }

                    self.visited[self.guard.y][self.guard.x].visit(&self.guard.direction);
                }
                Movement::HitObstacle => {
                    self.guard.direction.rotate();
                    self.visited[self.guard.y][self.guard.x].visit(&self.guard.direction);
                }
                Movement::OutOfMap => {
                    break;
                }
            }
        }

        (visited_count, loop_obstacles_count)
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (y, row) in self.obstacles.iter().enumerate() {
            for (x, &obstacle) in row.iter().enumerate() {
                if self.guard.x == x && self.guard.y == y {
                    write!(f, "{}", self.guard.direction)?;
                } else if obstacle {
                    write!(f, "#")?;
                } else if self.potential_obstacle == Some((x, y)) {
                    write!(f, "O")?;
                } else if self.visited[y][x].is_visited() {
                    if (self.visited[y][x].has_direction(&Direction::Up)
                        || self.visited[y][x].has_direction(&Direction::Down))
                        && !(self.visited[y][x].has_direction(&Direction::Left)
                            || self.visited[y][x].has_direction(&Direction::Right))
                    {
                        write!(f, "|")?;
                    } else if (self.visited[y][x].has_direction(&Direction::Left)
                        || self.visited[y][x].has_direction(&Direction::Right))
                        && !(self.visited[y][x].has_direction(&Direction::Up)
                            || self.visited[y][x].has_direction(&Direction::Down))
                    {
                        write!(f, "-")?;
                    } else {
                        write!(f, "+")?;
                    }
                } else {
                    write!(f, ".")?;
                }
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

fn calculate_guard_route(file_name: &str) -> Result<(u64, u64)> {
    let mut map = Map::from_file(file_name)?;

    Ok(map.patrol())
}

fn main() -> Result<()> {
    let args = Args::parse();

    let (visited_count, loop_obstructions) = calculate_guard_route(&args.file)?;

    println!("Distinct positions: {visited_count}");
    println!("Potential obstructions that cause a loop: {loop_obstructions}");

    Ok(())
}
