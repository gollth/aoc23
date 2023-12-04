use std::{collections::HashMap, fs, str::FromStr};

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
            // '#' => CharKind::Symbol,
            // _ => CharKind::Ignore,
            _ => CharKind::Symbol,
        }
    }
}

impl FromStr for Schematic {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut symbols = HashMap::new();
        let mut numbers = HashMap::new();
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
                            let (x, y) = (x as i32, y as i32);
                            symbols.extend(
                                ((x - 1)..=(x + 1))
                                    .cartesian_product((y - 1)..=(y + 1))
                                    .map(|(x, y)| (Coord::new(x, y), symbol)),
                            );
                        }
                        CharKind::Digit => {
                            let (x, a) = group.next().expect("Number");
                            let mut s = String::from(a);
                            s.extend(group.map(|(_, c)| c));
                            let value = s.parse().expect(&format!("Valid number, not {s}"));
                            numbers.insert(Coord::new(x as i32, y as i32), value);
                        }
                    }
                }
            })
            .collect::<Vec<_>>();
        Ok(Schematic { numbers, symbols })
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
}

fn main() -> anyhow::Result<()> {
    let args = Options::parse();
    let solution = match args.part {
        Part::One => Schematic::from_str(&fs::read_to_string(&args.input)?)?
            .numbers_touching_symbol()
            .sum::<u32>(),
        Part::Two => todo!(),
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
}
