use std::{fmt::Debug, str::FromStr};

use aoc23::Part;

use anyhow::Result;
use clap::Parser;
use ndarray::prelude::*;

/// Day 13: Point of Incidence
#[derive(Debug, Parser)]
struct Options {
    /// Path to the file with the input data
    #[clap(short, long, default_value = "sample/thirteenth.txt")]
    input: String,

    /// Which part of the day to solve
    part: Part,
}

fn main() -> anyhow::Result<()> {
    let args = Options::parse();
    let input = std::fs::read_to_string(args.input)?;
    let grids = input
        .split("\n\n")
        .map(|section| Grid::from_str(section))
        .collect::<Result<Vec<_>>>()?;
    let solution = match args.part {
        Part::One => {
            grids
                .iter()
                .map(|grid| grid.fold_line_vertical().expect("vertical fold"))
                .sum::<usize>()
                + grids
                    .iter()
                    .map(|grid| grid.fold_line_horizontal().expect("horizontal fold"))
                    .sum::<usize>()
                    * 100
        }
        Part::Two => unimplemented!(),
    };

    println!("Solution part {:?}: {solution}", args.part);

    Ok(())
}

#[derive(PartialEq, Eq)]
struct Grid(Array2<char>);

impl Grid {
    fn fold_line_horizontal(&self) -> Option<usize> {
        let rows = self.0.nrows();
        let middle = rows / 2;
        for fold in 1..rows - 1 {
            let n = if fold <= middle { fold } else { rows - fold };
            let above = self.0.slice(s![(fold-n)..fold;-1, ..]);
            let below = self.0.slice(s![fold..(fold + n), ..]);

            if above == below {
                return Some(fold);
            }
            // println!("{:?}", Grid(above.to_owned()));
            // println!("{:?}", Grid(below.to_owned()));
        }
        None
    }
    fn fold_line_vertical(&self) -> Option<usize> {
        let cols = self.0.ncols();
        let middle = cols / 2;
        for fold in 1..cols - 1 {
            let n = if fold <= middle { fold } else { cols - fold };
            let left = self.0.slice(s![.., (fold-n)..fold;-1]);
            let right = self.0.slice(s![.., fold..(fold + n)]);
            if left == right {
                return Some(fold);
            }
            // println!("{:?}", Grid(left.to_owned()));
            // println!("{:?}", Grid(right.to_owned()));
        }
        None
    }
}

impl FromStr for Grid {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let two_d = (s.lines().count(), s.lines().next().unwrap_or("").len());
        let grid = Array::from_iter(
            s.replace('#', "█")
                .replace('.', "·")
                .lines()
                .flat_map(|line| line.chars()),
        );
        Ok(Grid(grid.into_shape(two_d)?))
    }
}

impl Debug for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.0.nrows() {
            for x in 0..self.0.ncols() {
                write!(f, "{}", self.0[[y, x]])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    fn sample_a_1() {
        let input = include_str!("../../sample/thirteenth.txt");
        let input = input.split("\n\n").nth(0).unwrap();
        let grid = Grid::from_str(input).expect("parsing");
        assert_eq!(Some(5), grid.fold_line_vertical(), "\n{grid:?}");
    }
    #[rstest]
    fn sample_a_2() {
        let input = include_str!("../../sample/thirteenth.txt");
        let input = input.split("\n\n").nth(1).unwrap();
        let grid = Grid::from_str(input).expect("parsing");
        assert_eq!(Some(4), grid.fold_line_horizontal(), "\n{grid:?}");
    }
}
