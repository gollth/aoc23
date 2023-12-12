use std::{collections::BTreeSet, fmt::Debug, str::FromStr};

use aoc23::Part;

use clap::Parser;
use euclid::Vector2D;
use itertools::Itertools;
use ndarray::prelude::*;

/// Day 10: Pipe Maze
#[derive(Debug, Parser)]
struct Options {
    /// Path to the file with the input data
    #[clap(short, long, default_value = "sample/eleventh.txt")]
    input: String,

    /// Which part of the day to solve
    part: Part,

    /// Print the universe to stdout
    #[clap(short, long)]
    verbose: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Options::parse();
    let input = std::fs::read_to_string(&args.input)?;

    let mut universe = Universe::from_str(&input)?;

    universe.expand(match args.part {
        Part::One => 2,
        Part::Two => 1_000_000,
    });

    let solution = universe
        .shortest_paths()
        .map(|(_, _, dist)| dist)
        .sum::<i64>();

    if args.verbose {
        println!("{universe:?}");
    }
    println!("Solution part {:?}: {solution}", args.part);
    Ok(())
}

#[derive(Default, PartialEq, Eq)]
struct Universe {
    sky: Array2<char>,
    expansion: usize,
    horizontal: BTreeSet<i64>,
    vertical: BTreeSet<i64>,
}

type Coord = Vector2D<i64, euclid::UnknownUnit>;

impl Debug for Universe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let shape = self.sky.shape();
        for y in 0..shape[0] {
            for x in 0..shape[1] {
                write!(f, "{}", self.sky[[y, x]])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Universe {
    fn expand(&mut self, factor: usize) {
        self.expansion = factor - 1;
        self.vertical = self
            .sky
            .axis_iter(Axis(0))
            .enumerate()
            .filter(|(_, row)| row.iter().all(|c| *c != GALAXY))
            .map(|(i, _)| i as i64)
            .collect::<BTreeSet<_>>();

        self.horizontal = self
            .sky
            .axis_iter(Axis(1))
            .enumerate()
            .filter(|(_, col)| col.iter().all(|c| *c != GALAXY))
            .map(|(i, _)| i as i64)
            .collect::<BTreeSet<_>>();
    }

    fn manhattan(&self, a: &Coord, b: &Coord) -> i64 {
        let (start, end) = (a.min(*b), a.max(*b));
        let eh = self.horizontal.range(start.x..end.x).count() * self.expansion;
        let ev = self.vertical.range(start.y..end.y).count() * self.expansion;
        let d = (*a - *b).abs();
        d.x + d.y + eh as i64 + ev as i64
    }

    fn galaxies(&self) -> impl Iterator<Item = Coord> + '_ + Clone {
        self.sky
            .indexed_iter()
            .filter(|(_, c)| **c == GALAXY)
            .map(|((x, y), _)| Coord::new(y as i64, x as i64))
    }

    fn shortest_paths(&self) -> impl Iterator<Item = (Coord, Coord, i64)> + '_ {
        self.galaxies()
            .tuple_combinations()
            .map(|(a, b)| (a, b, self.manhattan(&a, &b)))
    }
}

impl FromStr for Universe {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let n = s.lines().count();
        let mut sky = Array2::from_elem((n, n), VOID);

        for (coord, _) in s
            .lines()
            .enumerate()
            .flat_map(|(y, line)| line.chars().enumerate().map(move |(x, c)| ([y, x], c)))
            .filter(|(_, c)| *c == '#')
        {
            sky[coord] = GALAXY;
        }
        Ok(Universe {
            sky,
            ..Default::default()
        })
    }
}

const VOID: char = '·';
const GALAXY: char = '●';

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case(2, 374)]
    #[case(10, 1030)]
    #[case(100, 8410)]
    fn sample(#[case] expansion: usize, #[case] expected_path_len: i64) {
        let input = include_str!("../../sample/eleventh.txt");
        let mut universe = Universe::from_str(input).expect("parsing");

        universe.expand(expansion);
        assert_eq!(
            expected_path_len,
            universe
                .shortest_paths()
                .map(|(_, _, dist)| dist)
                .sum::<i64>(),
            "{universe:?}"
        );
    }
}
