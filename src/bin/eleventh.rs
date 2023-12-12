use std::{fmt::Debug, str::FromStr};

use aoc23::Part;

use clap::Parser;
use euclid::Vector2D;
use itertools::Itertools;
use ndarray::{concatenate, prelude::*};

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

    universe.expand();
    let solution = match args.part {
        Part::One => universe
            .shortest_paths()
            .map(|(_, _, dist)| dist)
            .sum::<i32>(),
        Part::Two => unimplemented!(),
    };

    if args.verbose {
        println!("{universe:?}");
    }
    println!("Solution part {:?}: {solution}", args.part);
    Ok(())
}

#[derive(Default, PartialEq, Eq)]
struct Universe {
    sky: Array2<char>,
    route: Route,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Route {
    path: Vec<Coord>,
    cost: i32,
}

type Coord = Vector2D<i32, euclid::UnknownUnit>;
type Coordf = Vector2D<f32, euclid::UnknownUnit>;

fn is_on_line(c: Coord, a: Coord, b: Coord) -> bool {
    let a = Coordf::new(a.x as f32, a.y as f32);
    let b = Coordf::new(b.x as f32, b.y as f32);
    let c = Coordf::new(c.x as f32, c.y as f32);
    let d = (a - c).length() + (c - b).length() - (a - b).length();
    d.abs() < 0.12
}

impl Debug for Universe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let shape = self.sky.shape();
        for y in 0..shape[0] {
            for x in 0..shape[1] {
                let c = Coord::new(y as i32, x as i32);
                let item = self.sky[[y, x]];
                let is_on_line = self
                    .route
                    .path
                    .iter()
                    .copied()
                    .tuple_windows()
                    .any(|(a, b)| is_on_line(c, a, b));
                if item != GALAXY && is_on_line {
                    write!(f, "{ROUTE}")?;
                } else {
                    write!(f, "{item}")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Universe {
    #[allow(dead_code)]
    fn dimension(&self) -> (usize, usize) {
        (self.sky.ncols(), self.sky.nrows())
    }
    fn add_row(&mut self, i: usize) {
        let row = Array2::from_elem((1, self.sky.shape()[1]), '┈');
        self.sky = concatenate![
            Axis(0),
            self.sky.slice(s![..i, ..]),
            row,
            self.sky.slice(s![i.., ..])
        ];
    }

    fn add_col(&mut self, i: usize) {
        let col = Array2::from_elem((self.sky.shape()[0], 1), '┊');
        self.sky = concatenate![
            Axis(1),
            self.sky.slice(s![.., ..i]),
            col,
            self.sky.slice(s![.., i..])
        ];
    }

    fn expand(&mut self) {
        let row_insertions = self
            .sky
            .axis_iter(Axis(0))
            .enumerate()
            .filter(|(_, row)| row.iter().all(|c| *c != GALAXY))
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        for (offset, i) in row_insertions.into_iter().enumerate() {
            self.add_row(i + offset);
        }

        let col_insertions = self
            .sky
            .axis_iter(Axis(1))
            .enumerate()
            .filter(|(_, col)| col.iter().all(|c| *c != GALAXY))
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        for (offset, i) in col_insertions.into_iter().enumerate() {
            self.add_col(i + offset);
        }
    }

    fn manhattan(a: &Coord, b: &Coord) -> i32 {
        let d = (*a - *b).abs();
        d.x + d.y
    }

    fn galaxies(&self) -> impl Iterator<Item = Coord> + '_ + Clone {
        self.sky
            .indexed_iter()
            .filter(|(_, c)| **c == GALAXY)
            .map(|((x, y), _)| Coord::new(x as i32, y as i32))
    }

    fn shortest_paths(&self) -> impl Iterator<Item = (Coord, Coord, i32)> + '_ {
        self.galaxies()
            .tuple_combinations()
            .map(|(a, b)| (a, b, Self::manhattan(&a, &b)))
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

const VOID: char = ' ';
const GALAXY: char = '●';
const ROUTE: char = '·';

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    fn sample_a() {
        let input = include_str!("../../sample/eleventh.txt");
        let mut universe = Universe::from_str(input).expect("parsing");

        assert_eq!((10, 10), universe.dimension(), "{universe:?}");

        universe.expand();
        println!("{universe:?}");
        assert_eq!((13, 12), universe.dimension(), "{universe:?}");
        assert_eq!(
            374,
            universe
                .shortest_paths()
                .map(|(_, _, dist)| dist)
                .sum::<i32>(),
            "{universe:?}"
        );
    }
}
