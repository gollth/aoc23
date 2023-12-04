use std::{
    collections::{HashMap, HashSet},
    fs,
    str::FromStr,
};

use aoc23::Part;
use clap::Parser;
use itertools::Itertools;

/// Day 3: Gear Ratios
#[derive(Parser)]
struct Options {
    /// Path to the file with the input data
    #[clap(short, long, default_value = "sample/third.txt")]
    input: String,

    /// Which part of the day to solve
    part: Part,
}

#[derive(Debug, Default)]
struct Schematic {
    symbols: HashMap<Coord, char>,
    gears: HashSet<Coord>,
    numbers: HashMap<Coord, u32>,
}

type Coord = euclid::Vector2D<i32, ()>;

#[derive(Debug, PartialEq, Eq)]
enum CharKind {
    Digit,
    Ignore,
    Symbol,
}
impl From<char> for CharKind {
    fn from(c: char) -> CharKind {
        match c {
            '0'..='9' => CharKind::Digit,
            '.' => CharKind::Ignore,
            _ => CharKind::Symbol,
        }
    }
}

fn neighbors(c: Coord) -> impl Iterator<Item = Coord> {
    ((c.x - 1)..=(c.x + 1))
        .cartesian_product((c.y - 1)..=(c.y + 1))
        .map(|(x, y)| Coord::new(x, y))
        .filter(move |n| *n != c)
}

impl FromStr for Schematic {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut symbols = HashMap::new();
        let mut numbers = HashMap::new();
        let mut gears = HashSet::new();
        let _ = s
            .lines()
            .enumerate()
            .map(|(y, line)| {
                for (kind, mut group) in line
                    .chars()
                    .enumerate()
                    .group_by(|(_, c)| CharKind::from(*c))
                    .into_iter()
                {
                    match kind {
                        CharKind::Ignore => {}
                        CharKind::Symbol => {
                            let (x, symbol) = group.next().expect("Symbol");
                            let c = Coord::new(x as i32, y as i32);
                            symbols.extend(neighbors(c).map(|c| (c, symbol)));
                            if symbol == '*' {
                                gears.insert(c);
                            }
                        }
                        CharKind::Digit => {
                            let (x, a) = group.next().expect("Number");
                            let mut s = String::from(a);
                            s.extend(group.map(|(_, c)| c));
                            let value = s
                                .parse()
                                .unwrap_or_else(|_| panic!("Valid number, not {s}"));
                            numbers.insert(Coord::new(x as i32, y as i32), value);
                        }
                    }
                }
            })
            .collect::<Vec<_>>();
        Ok(Schematic {
            numbers,
            symbols,
            gears,
        })
    }
}

impl Schematic {
    fn numbers_touching_symbol(&self) -> impl Iterator<Item = u32> + '_ {
        self.numbers
            .iter()
            .filter(|(coord, n)| {
                (0..n.to_string().len())
                    .map(|x| **coord + Coord::new(x as i32, 0))
                    .any(|coord| self.symbols.contains_key(&coord))
            })
            .map(|(_, n)| *n)
    }

    fn gear_ratios(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        self.gears.iter().filter_map(|gc| {
            self.numbers
                .iter()
                .filter(|(nc, num)| {
                    neighbors(*gc)
                        .cartesian_product(
                            (0..format!("{num}").len()).map(|x| **nc + Coord::new(x as i32, 0)),
                        )
                        .any(|(gc, nc)| gc == nc)
                })
                .map(|(_, num)| *num)
                .next_tuple()
        })
    }
}

fn main() -> anyhow::Result<()> {
    let args = Options::parse();
    let schematic = Schematic::from_str(&fs::read_to_string(&args.input)?)?;
    let solution = match args.part {
        Part::One => schematic.numbers_touching_symbol().sum::<u32>(),
        Part::Two => schematic.gear_ratios().map(|(a, b)| a * b).sum::<u32>(),
    };
    println!("Solution part {:?}: {solution}", args.part);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_part_one() {
        let input = include_str!("../../sample/third.txt");
        assert_eq!(
            4361,
            Schematic::from_str(input)
                .expect("Schematic FromStr")
                .numbers_touching_symbol()
                .sum::<u32>()
        )
    }

    #[test]
    fn sample_part_two() {
        let input = include_str!("../../sample/third.txt");
        assert_eq!(
            467835,
            Schematic::from_str(input)
                .expect("Schematic FromStr")
                .gear_ratios()
                .map(|(a, b)| a * b)
                .sum::<u32>()
        )
    }
}
