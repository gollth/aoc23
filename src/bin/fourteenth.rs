use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    ops::Not,
    str::FromStr,
};

use aoc23::{cycle, Part};

use anyhow::{anyhow, Result};
use clap::Parser;
use itertools::Itertools;
use termion::color::{Fg, Reset, Rgb, Yellow};

/// Day 14: Parabolic Reflector Dish
#[derive(Debug, Parser)]
struct Options {
    /// Path to the file with the input data
    #[clap(short, long, default_value = "sample/fourteenth.txt")]
    input: String,

    /// Which part of the day to solve
    part: Part,
}

fn main() -> Result<()> {
    let args = Options::parse();
    let input = std::fs::read_to_string(args.input)?;
    let mut platform = Platform::from_str(&input)?;

    let mut states = Vec::new();

    let solution = match args.part {
        Part::One => {
            platform.tilt(NORTH);
            platform.total_north_load()
        }
        Part::Two => {
            let until = loop {
                for dir in CYCLE.iter() {
                    platform.tilt(*dir);
                }
                states.push(platform.total_north_load());

                if let Some((mu, lambda)) = cycle(states.iter()) {
                    break ((1_000_000_000 - mu) % lambda) + mu;
                }
            };

            // Reset
            platform = Platform::from_str(&input)?;
            for _ in 0..until {
                for dir in CYCLE.iter() {
                    platform.tilt(*dir);
                }
            }
            platform.total_north_load()
        }
    };

    println!("Solution part {:?} {solution}", args.part);
    Ok(())
}

type Coord = euclid::Vector2D<i32, euclid::UnknownUnit>;
const NORTH: Coord = Coord::new(0, -1);
const SOUTH: Coord = Coord::new(0, 1);
const EAST: Coord = Coord::new(1, 0);
const WEST: Coord = Coord::new(-1, 0);

const CYCLE: [Coord; 4] = [NORTH, WEST, SOUTH, EAST];

#[derive(Debug, Clone)]
struct Platform {
    rocks: HashMap<Coord, Rock>,
    nrows: i32,
    ncols: i32,
}

impl PartialEq for Platform {
    fn eq(&self, other: &Self) -> bool {
        self.ncols == other.ncols
            && self.nrows == other.nrows
            && self.round_rocks() == other.round_rocks()
    }
}

#[derive(Default, Debug, PartialEq, Copy, Clone, Eq)]
enum Rock {
    #[default]
    None,
    Round,
    Square,
}

impl Platform {
    fn get(&self, c: Coord) -> Rock {
        if c.x < 0 || self.ncols <= c.x || c.y < 0 || self.nrows <= c.y {
            return Rock::Square;
        }
        self.rocks.get(&c).copied().unwrap_or_default()
    }

    fn outer(&self, dir: Coord) -> i32 {
        if dir == NORTH || dir == SOUTH {
            return self.ncols;
        }
        if dir == EAST || dir == WEST {
            return self.nrows;
        }
        panic!("Only N,S,W or E directions supported")
    }

    fn inner_iter(&self, dir: Coord) -> Box<dyn Iterator<Item = i32>> {
        if dir == NORTH {
            Box::new(-1..=self.nrows)
        } else if dir == SOUTH {
            Box::new((-1..=self.nrows).rev())
        } else if dir == EAST {
            Box::new((-1..=self.ncols).rev())
        } else if dir == WEST {
            Box::new(-1..=self.ncols)
        } else {
            panic!("Only N,S,W or E directions supported")
        }
    }

    fn coord(&self, dir: Coord, outer: i32, inner: i32) -> Coord {
        if dir == NORTH || dir == SOUTH {
            Coord::new(outer, inner)
        } else if dir == EAST || dir == WEST {
            Coord::new(inner, outer)
        } else {
            panic!("Only N,S,W or E directions supported")
        }
    }

    fn tilt(&mut self, dir: Coord) {
        let mut rocks = HashMap::new();
        for outer in 0..self.outer(dir) {
            let new_coords = self
                .inner_iter(dir)
                .map(|inner| self.coord(dir, outer, inner))
                .map(|c| (c, self.get(c)))
                .group_by(|(_, r)| r == &Rock::Square)
                .into_iter()
                .filter_map(|(is_square, region)| is_square.not().then_some(region))
                .filter_map(|region| {
                    let mut region = region.peekable();
                    region.peek().copied().map(|(start, _)| {
                        (
                            start,
                            region.filter(|(_, rock)| rock == &Rock::Round).count(),
                        )
                    })
                })
                .filter(|(_, n)| *n > 0)
                .flat_map(move |(start, n)| (0..).map(move |i| start - dir * i).take(n))
                .map(|coord| (coord, Rock::Round))
                .collect::<HashMap<_, _>>();
            rocks.extend(new_coords);
        }
        self.rocks.retain(|_, rock| rock != &Rock::Round);
        self.rocks.extend(rocks);
    }

    fn total_north_load(&self) -> i32 {
        self.rocks
            .iter()
            .filter(|(_, item)| item == &&Rock::Round)
            .map(|(coord, _)| self.nrows - coord.y)
            .sum()
    }
    fn round_rocks(&self) -> HashSet<Coord> {
        self.rocks
            .iter()
            .filter(|(_, rock)| rock == &&Rock::Round)
            .map(|(coord, _)| coord)
            .copied()
            .collect()
    }
}

impl FromStr for Platform {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let rocks = s
            .lines()
            .enumerate()
            .flat_map(|(y, line)| {
                line.trim().chars().enumerate().map(move |(x, c)| {
                    Ok::<(Coord, Rock), anyhow::Error>((
                        Coord::new(x as i32, y as i32),
                        Rock::try_from(c)?,
                    ))
                })
            })
            .process_results(|iter| iter.collect::<HashMap<_, _>>())?;
        if rocks.is_empty() {
            return Err(anyhow!("Empty platforms not allowed"));
        }
        let ncols = rocks.keys().map(|i| i.x).max().unwrap_or_default() + 1;
        let nrows = rocks.keys().map(|i| i.y).max().unwrap_or_default() + 1;
        Ok(Self {
            rocks,
            ncols,
            nrows,
        })
    }
}

impl Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "╭")?;
        for _ in 0..self.ncols + 2 {
            write!(f, "─")?;
        }
        writeln!(f, "╮")?;
        for y in -1..=self.nrows {
            write!(f, "│")?;
            for x in -1..=self.ncols {
                let coord = Coord::new(x, y);
                let rock = self.get(coord);
                if rock == Rock::Square {
                    write!(f, "{}", Fg(Rgb(160, 160, 160)))?;
                } else if rock == Rock::Round {
                    write!(f, "{}", Fg(Yellow))?;
                }
                write!(f, "{}", rock)?;
                write!(f, "{}", Fg(Reset))?;
            }
            writeln!(f, "│")?;
        }
        write!(f, "╰")?;
        for _ in 0..self.ncols + 2 {
            write!(f, "─")?;
        }
        write!(f, "╯")?;
        Ok(())
    }
}

impl TryFrom<char> for Rock {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(Rock::None),
            'O' => Ok(Rock::Round),
            '#' => Ok(Rock::Square),
            _ => Err(anyhow!("Unknown rock: {value}")),
        }
    }
}
impl Display for Rock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => '·',
                Self::Round => '●',
                Self::Square => '▧',
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    fn sample_a() {
        let input = include_str!("../../sample/fourteenth.txt");
        let mut platform = Platform::from_str(input).expect("parsing");

        platform.tilt(NORTH);
        assert_eq!(136, platform.total_north_load(), "Platform:\n{platform}");
    }

    #[rstest]
    #[case(
        NORTH,
        "OOOO.#.O..
         OO..#....#
         OO..O##..O
         O..#.OO...
         ........#.
         ..#....#.#
         ..O..#.O.O
         ..O.......
         #....###..
         #....#...."
    )]
    #[case(
        SOUTH,
        ".....#....
         ....#....#
         ...O.##...
         ...#......
         O.O....O#O
         O.#..O.#.#
         O....#....
         OO....OO..
         #OO..###..
         #OO.O#...O"
    )]
    #[case(
        EAST,
        "....O#....
         .OOO#....#
         .....##...
         .OO#....OO
         ......OO#.
         .O#...O#.#
         ....O#..OO
         .........O
         #....###..
         #..OO#...."
    )]
    #[case(
        WEST,
        "O....#....
         OOO.#....#
         .....##...
         OO.#OO....
         OO......#.
         O.#O...#.#
         O....#OO..
         O.........
         #....###..
         #OO..#...."
    )]
    fn sample_a_manual(#[case] tilt_dir: Coord, #[case] expected: Platform) {
        let input = include_str!("../../sample/fourteenth.txt");
        let mut platform = Platform::from_str(input).expect("parsing");

        platform.tilt(tilt_dir);
        assert_eq!(
            expected.round_rocks(),
            platform.round_rocks(),
            "Platform:\n{platform}\n\nExpected\n{expected}"
        );
    }

    #[rstest]
    #[case(
        1,
        ".....#....
         ....#...O#
         ...OO##...
         .OO#......
         .....OOO#.
         .O#...O#.#
         ....O#....
         ......OOOO
         #...O###..
         #..OO#...."
    )]
    #[case(
        2,
        ".....#....
         ....#...O#
         .....##...
         ..O#......
         .....OOO#.
         .O#...O#.#
         ....O#...O
         .......OOO
         #..OO###..
         #.OOO#...O"
    )]
    #[case(
        3,
        ".....#....
         ....#...O#
         .....##...
         ..O#......
         .....OOO#.
         .O#...O#.#
         ....O#...O
         .......OOO
         #...O###.O
         #.OOO#...O"
    )]
    fn sample_b_manual(#[case] cycles: usize, #[case] expected: Platform) {
        let input = include_str!("../../sample/fourteenth.txt");
        let mut platform = Platform::from_str(input).expect("parsing");

        for dir in CYCLE.iter().cycle().take(CYCLE.len() * cycles) {
            platform.tilt(*dir);
        }
        assert_eq!(
            expected, platform,
            "Platform:\n{platform}\n\nExpected\n{expected}"
        );
    }
}
