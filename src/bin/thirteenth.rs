use std::{fmt::Debug, str::FromStr};

use aoc23::Part;

use anyhow::Result;
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
    let mut grids = input
        .split("\n\n")
        .map(Grid::from_str)
        .collect::<Result<Vec<_>>>()?;

    let mut lefts = 0;
    let mut aboves = 0;

    if args.part == Part::Two {
        for grid in grids.iter_mut() {
            let (_index, fold, dir) = [Reflection::Horizontal, Reflection::Vertical]
                .into_iter()
                .flat_map(|r| grid.find_smudge(r))
                .next()
                .expect("a smudge");
            match dir {
                Reflection::Horizontal => aboves += fold,
                Reflection::Vertical => lefts += fold,
            }
        }
    } else {
        for (dir, x) in grids.iter().flat_map(|grid| {
            grid.fold_line(Reflection::Horizontal)
                .or(grid.fold_line(Reflection::Vertical))
        }) {
            match dir {
                Reflection::Vertical => lefts += x,
                Reflection::Horizontal => aboves += x,
            }
        }
    }
    let solution = lefts + 100 * aboves;
    println!("Solution part {:?}: {solution}", args.part);

    Ok(())
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Reflection {
    Horizontal,
    Vertical,
}

#[derive(PartialEq, Eq, Clone)]
struct Grid(Array2<i8>);

impl Grid {
    fn split(&self, fold: usize, direction: Reflection) -> (ArrayView2<i8>, ArrayView2<i8>) {
        let n = self.end(direction);

        let k = if fold <= n / 2 { fold } else { n - fold };
        match direction {
            Reflection::Vertical => (
                self.0.slice(s![.., (fold-k)..fold;-1]),
                self.0.slice(s![.., fold..(fold + k)]),
            ),
            Reflection::Horizontal => (
                self.0.slice(s![(fold-k)..fold;-1, ..]),
                self.0.slice(s![fold..(fold + k), ..]),
            ),
        }
    }

    fn end(&self, direction: Reflection) -> usize {
        match direction {
            Reflection::Horizontal => self.0.nrows(),
            Reflection::Vertical => self.0.ncols(),
        }
    }

    fn find_smudge(&self, direction: Reflection) -> Option<((usize, usize), usize, Reflection)> {
        (1..self.end(direction))
            .filter_map(|fold| {
                let (a, b) = self.split(fold, direction);
                (&a - &b)
                    .indexed_iter()
                    .filter(|(_, elem)| elem.abs() == 1)
                    .map(|((row, col), _)| {
                        (
                            match direction {
                                Reflection::Horizontal => (fold - 1 - row, col),
                                Reflection::Vertical => (row, fold - col - 1),
                            },
                            fold,
                            direction,
                        )
                    })
                    .exactly_one()
                    .ok()
            })
            .next()
    }

    fn fold_line(&self, direction: Reflection) -> Option<(Reflection, usize)> {
        match direction {
            Reflection::Horizontal => self.fold_line_horizontal(),
            Reflection::Vertical => self.fold_line_vertical(),
        }
        .map(|i| (direction, i))
    }
    fn fold_line_horizontal(&self) -> Option<usize> {
        (1..self.0.nrows()).find(|fold| {
            let (above, below) = self.split(*fold, Reflection::Horizontal);
            above == below
        })
    }
    fn fold_line_vertical(&self) -> Option<usize> {
        (1..self.0.ncols()).find(|fold| {
            let (left, right) = self.split(*fold, Reflection::Vertical);
            left == right
        })
    }
}

const BOX: char = '█';
const EMPTY: char = '·';

impl FromStr for Grid {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let two_d = (s.lines().count(), s.lines().next().unwrap_or("").len());
        let grid = Array::from_iter(
            s.replace('#', &BOX.to_string())
                .replace('.', &EMPTY.to_string())
                .lines()
                .flat_map(|line| {
                    line.trim().chars().map(|c| match c {
                        BOX => 1,
                        EMPTY => 0,
                        _ => panic!("Unknown character for Grid: {c} only {BOX} & {EMPTY} allowed"),
                    })
                }),
        );
        Ok(Grid(grid.into_shape(two_d)?))
    }
}

impl Debug for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.0.nrows() {
            for x in 0..self.0.ncols() {
                write!(f, "{}", if self.0[[y, x]] == 1 { BOX } else { EMPTY })?;
            }
            if y == self.0.nrows() - 1 {
                continue;
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
        #[case] grid: Grid,
    ) {
        // let grid = Grid::from_str(s).expect("parsing");
        let actual = match reflection {
            Reflection::Horizontal => grid.fold_line_horizontal(),
            Reflection::Vertical => grid.fold_line_vertical(),
        };
        assert_eq!(Some(expected_split), actual, "\n{grid:?}");
    }

    #[rstest]
    #[case(
        Reflection::Horizontal,
        Some(((0,0),3)),
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
        Some(((0,4),1)),
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
        Reflection::Horizontal,
        Some(((0,3),1)),
        "
            .#.##
            .#..#"
    )]
    #[case(
        Reflection::Horizontal,
        Some(((1,0), 2)),
        "
            .###.
            .#..#
            ##..#
            .###."
    )]
    #[case(
        Reflection::Horizontal,
        Some(((2,3),4)),
        "
            .###.
            .#..#
            .##..
            ##..#
            ##..#
            .###."
    )]
    #[case(
        Reflection::Vertical,
        Some(((0,2), 3)),
        "
            .#.##
            .#..#"
    )]
    #[case(
        Reflection::Vertical,
        Some(((3,2), 4)),
        "
            ...##.
            ..#..#
            .##..#
            #..###"
    )]
    #[case(
        Reflection::Vertical,
        None,
        "
            .#....#..#...
            #..##..##....
            ...##...#.##.
            ...##...###.#
            .##..##.#.#.#
            .##..##.#.###
            ...##...###.#"
    )]
    #[case(
        Reflection::Horizontal,
        Some(((4,11), 5)),
        "
            .#....#..#...
            #..##..##....
            ...##...#.##.
            ...##...###.#
            .##..##.#.#.#
            .##..##.#.###
            ...##...###.#"
    )]
    #[case(
        Reflection::Horizontal,
        Some(((6,7), 7)),
        "
            #...##.#.##
            #...##.#.##
            .###..#.#.#
            ##.###..#..
            ..#...##..#
            ....##.####
            ##.###.#.#.
            ##.###...#.
            ....##.####
            ..#...##..#
            ##.###..#..
            .###..#.#.#
            #...##.#.##"
    )]
    fn sample_b_manual(
        #[case] reflection: Reflection,
        #[case] expectation: Option<((usize, usize), usize)>,
        #[case] grid: Grid,
    ) {
        let expectation = expectation.map(|(a, b)| (a, b, reflection));
        assert_eq!(
            expectation,
            grid.find_smudge(reflection),
            "split {reflection:?}: \n{:?}",
            grid
        );
    }

    #[rstest]
    fn sample_b() {
        let input = include_str!("../../sample/thirteenth.txt");

        let mut grids = input
            .split("\n\n")
            .map(Grid::from_str)
            .collect::<Result<Vec<_>>>()
            .unwrap();

        let mut lefts = 0;
        let mut aboves = 0;
        for grid in grids.iter_mut() {
            let (_index, fold, dir) = [Reflection::Horizontal, Reflection::Vertical]
                .into_iter()
                .flat_map(|r| grid.find_smudge(r))
                .next()
                .expect("a smudge");
            match dir {
                Reflection::Vertical => lefts += fold,
                Reflection::Horizontal => aboves += fold,
            };
        }

        assert_eq!(400, lefts + 100 * aboves);
    }
}
