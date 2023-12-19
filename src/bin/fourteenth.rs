use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    ops::Not,
    str::FromStr,
};

use aoc23::Part;

use anyhow::{anyhow, Result};
use clap::Parser;
use itertools::Itertools;
use termion::color::{Fg, LightBlue, Reset, Rgb, Yellow};

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
    let north = Coord::new(0, -1);

    platform.tilt(north);
    println!("{platform}");
    println!("Solution: {}", platform.total_north_load());

    Ok(())
}

type Coord = euclid::Vector2D<i32, euclid::UnknownUnit>;

struct Platform {
    rocks: HashMap<Coord, Rock>,
    settled: HashSet<Coord>,
    nrows: i32,
    ncols: i32,
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

    fn tilt(&mut self, dir: Coord) {
        let mut rocks = HashMap::new();
        for col in 0..self.ncols {
            let new_coords = (-1..self.nrows)
                .map(move |y| Coord::new(col, y))
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
            settled: HashSet::new(),
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
                if self.settled.contains(&coord) {
                    write!(f, "{}", Fg(LightBlue))?;
                } else if rock == Rock::Square {
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

        let north = Coord::new(0, -1);

        platform.tilt(north);
        assert_eq!(136, platform.total_north_load(), "Platform:\n{platform}");
    }

    #[rstest]
    #[case(
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
    fn sample_a_manual(#[case] platform: Platform) {
        assert_eq!(136, platform.total_north_load(), "Platform:\n{platform}");
    }
}
