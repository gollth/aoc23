pub mod animation;

use anyhow::Result;
use itertools::Itertools;
use ndarray::prelude::*;
use std::{fmt::Debug, ops::Index, str::FromStr};

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum Reflection {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(PartialEq, Eq, Clone)]
pub struct Grid(Array2<i8>);

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

    fn rows(&self) -> usize {
        self.0.nrows()
    }
    fn cols(&self) -> usize {
        self.0.ncols()
    }

    fn end(&self, direction: Reflection) -> usize {
        match direction {
            Reflection::Horizontal => self.0.nrows(),
            Reflection::Vertical => self.0.ncols(),
        }
    }

    pub fn find_smudge(
        &self,
        direction: Reflection,
    ) -> Option<((usize, usize), usize, Reflection)> {
        (1..self.end(direction)).find_map(|fold| {
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
    }

    pub fn fold_line(&self, direction: Reflection) -> Option<(Reflection, usize)> {
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

impl Index<[usize; 2]> for Grid {
    type Output = i8;

    fn index(&self, index: [usize; 2]) -> &Self::Output {
        &self.0[index]
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
