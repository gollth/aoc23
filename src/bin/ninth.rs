use aoc23::Part;

use clap::Parser;
use itertools::Itertools;
use num::Zero;
use std::ops::Sub;
use std::str::FromStr;
use std::{fmt::Debug, rc::Rc, sync::Mutex};

/// Day 9: Mirage Maintenance
#[derive(Debug, Parser)]
struct Options {
    /// Path to the file with the input data
    #[clap(short, long, default_value = "sample/ninth.txt")]
    input: String,

    /// Which part of the day to solve
    part: Part,
}

fn main() -> anyhow::Result<()> {
    let args = Options::parse();
    let input = std::fs::read_to_string(&args.input)?;

    let solution = match args.part {
        Part::One => predict::<i64>(&input)
            .map(|history| history.sum::<i64>())
            .sum::<i64>(),
        Part::Two => unimplemented!(),
    };
    println!("Solution part {:?}: {solution:?}", args.part);
    Ok(())
}

fn predict<'a, T>(input: &'a str) -> impl Iterator<Item = PredictIter<'a, T>>
where
    T: Copy + Sub<Output = T> + Zero + 'a + FromStr,
{
    input
        .lines()
        .map(|line| {
            line.split_whitespace()
                .filter_map(|item| item.parse::<T>().ok())
        })
        .map(|values| PredictIter::new(values.rev()))
}

#[derive(Clone)]
struct PredictIter<'a, T> {
    first: Option<T>,
    diff: DiffIter<'a, T>,
}

impl<'a, T> PredictIter<'a, T>
where
    T: Copy + Sub<Output = T> + Zero + 'a,
{
    fn new<I: Iterator<Item = T> + 'a + Clone>(iter: I) -> Self {
        let mut iter = iter.peekable();
        let first = iter.peek().copied();
        Self {
            first,
            diff: DiffIter::new(iter),
        }
    }
}

impl<'a, T> Iterator for PredictIter<'a, T>
where
    T: Copy + Sub<Output = T> + PartialEq<T> + Zero + Debug + 'a,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.first.take().or_else(|| {
            let mut iter = self.diff.clone().peekable();
            let n = iter.peek().copied();
            self.diff = DiffIter::new(iter);
            n
        })
    }
}

#[derive(Clone)]
struct DiffIter<'a, T> {
    values: Rc<Mutex<dyn Iterator<Item = T> + 'a>>,
}
impl<'a, T> DiffIter<'a, T>
where
    T: Copy + Sub<Output = T> + Zero + 'a,
{
    fn new<I: Iterator<Item = T> + 'a + Clone>(iter: I) -> Self {
        Self {
            values: Rc::new(Mutex::new(iter.tuple_windows().map(|(a, b)| a - b))),
        }
    }
}
impl<'a, T> Iterator for DiffIter<'a, T>
where
    T: Sub<Output = T> + Copy,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.values.lock().expect("mutex").next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case(0, vec![15, 3, 0, 0, 0, 0])]
    #[case(1, vec![21, 6, 1, 0, 0, 0])]
    #[case(2, vec![45, 15, 6, 2, 0, 0])]
    fn sample_a_manual(#[case] line: usize, #[case] expectation: Vec<i32>) {
        let input = include_str!("../../sample/ninth.txt");
        let oasis = predict(input)
            .nth(line)
            .expect("input to contain line number {line}");
        assert_eq!(expectation, oasis.collect::<Vec<i32>>());
    }

    #[rstest]
    fn sample_a() {
        let input = include_str!("../../sample/ninth.txt");
        let oasis = predict::<i32>(input)
            .map(|history| history.sum::<i32>())
            .sum::<i32>();
        assert_eq!(114, oasis);
    }
}
