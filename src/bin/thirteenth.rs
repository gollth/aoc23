use std::{fmt::Debug, str::FromStr};

use aoc23::Part;

use anyhow::{anyhow, Result};
use clap::Parser;
use itertools::Itertools;
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
        .map(Grid::from_str)
        .collect::<Result<Vec<_>>>()?;
    let solution = match args.part {
        Part::One => {
            let (lefts, aboves) = grids
                .iter()
                .map(|grid| (grid, grid.fold_line_vertical(), grid.fold_line_horizontal()))
                .map(|(grid, left, above)| {
                    left.or(above)
                        .and(Some((left.unwrap_or_default(), above.unwrap_or_default())))
                        .ok_or(anyhow!(
                            "Grid does neither contain a vertical nor horizontal reflection:\n{grid:?}"
                        ))
                })
                .process_results(|values| {
                    values.fold((0, 0), |(al, aa), (il, ia)| (al + il, aa + ia))
                })?;
            lefts + 100 * aboves
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
        for fold in 1..rows {
            let n = if fold <= middle { fold } else { rows - fold };
            let above = self.0.slice(s![(fold-n)..fold;-1, ..]);
            let below = self.0.slice(s![fold..(fold + n), ..]);

            if above == below {
                return Some(fold);
            }
        }
        None
    }
    fn fold_line_vertical(&self) -> Option<usize> {
        let cols = self.0.ncols();
        let middle = cols / 2;
        for fold in 1..cols {
            let n = if fold <= middle { fold } else { cols - fold };
            let left = self.0.slice(s![.., (fold-n)..fold;-1]);
            let right = self.0.slice(s![.., fold..(fold + n)]);
            if left == right {
                return Some(fold);
            }
        }
        None
    }
}

impl FromStr for Grid {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let two_d = (s.lines().count(), s.lines().next().unwrap_or("").len());
        let grid = Array::from_iter(
            s.replace('#', "█")
                .replace('.', "·")
                .lines()
                .flat_map(|line| line.trim().chars()),
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

    #[derive(PartialEq, Eq)]
    enum Reflection {
        Horizontal,
        Vertical,
    }

    #[rstest]
    #[case(
        Reflection::Vertical,
        5,
        "
           #.##..##.
           ..#.##.#.
           ##......#
           ##......#
           ..#.##.#.
           ..##..##.
           #.#.##.#."
    )]
    #[case(
        Reflection::Horizontal,
        4,
        "
           #...##..#
           #....#..#
           ..##..###
           #####.##.
           #####.##.
           ..##..###
           #....#..#"
    )]
    #[case(
        Reflection::Vertical,
        7,
        "
           #.#######
           ....#####
           ..##.....
           .#.######
           .#..#....
           #........
           ..#...##.
           #.###....
           ###.##..#
           .########
           #.##.#..#
           .#.###..#
           ..###....
           ..###....
           ...###..#"
    )]
    #[case(
        Reflection::Vertical,
        16,
        "
            #.##.######.##.##
            ###...####...####
            ....##.##.##.....
            #..#.#....#.#..##
            .....##..##......
            #.#.###..###.#.##
            .##.#.####.#.#...
            .#..#......#..#..
            .####.####.####..
            ###...####...####
            #.##........##.##
            .#....#..#....#..
            ..###.####.###...
"
    )]
    fn sample_a_manual(
        #[case] reflection: Reflection,
        #[case] expected_split: usize,
        #[case] s: &str,
    ) {
        let grid = Grid::from_str(s).expect("parsing");
        let actual = match reflection {
            Reflection::Horizontal => grid.fold_line_horizontal(),
            Reflection::Vertical => grid.fold_line_vertical(),
        };
        assert_eq!(Some(expected_split), actual, "\n{grid:?}");
    }
}
