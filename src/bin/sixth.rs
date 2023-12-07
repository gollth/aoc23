use std::str::FromStr;

use aoc23::Part;

use anyhow::anyhow;
use clap::Parser;
use itertools::izip;
use nom::{
    bytes::complete::tag,
    character::complete::{newline, space1, u32},
    multi::separated_list1,
    sequence::{preceded, separated_pair, tuple},
    Finish, IResult, Parser as NomParser,
};

/// Day 6: Wait For It
#[derive(Debug, Parser)]
struct Options {
    /// Path to the file with the input data
    #[clap(short, long, default_value = "sample/sixth.txt")]
    input: String,

    /// Which part of the day to solve
    part: Part,
}

fn main() -> anyhow::Result<()> {
    let args = Options::parse();
    let input = std::fs::read_to_string(args.input)?;
    let races = Document::from_str(&input)?;

    let solution = match args.part {
        Part::One => races.margin(),
        Part::Two => unimplemented!(),
    };
    println!("Solution part {part:?}: {solution}", part = args.part);

    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct Race {
    time: u32,
    distance: u32,
}
impl Race {
    fn new(time: u32, distance: u32) -> Self {
        Self { time, distance }
    }
}

#[derive(Debug)]
struct Document(Vec<Race>);

impl FromStr for Document {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(parse_races(s).finish().map_err(|e| anyhow!("{e}"))?.1)
    }
}

impl Document {
    fn margin(&self) -> usize {
        self.0
            .iter()
            .map(|race| race.winning_charge().count())
            .product()
    }
}
impl Race {
    fn winning_charge(&self) -> impl Iterator<Item = Race> + '_ {
        (1..self.time)
            .map(|t| Race::new(t, (self.time - t).max(0) * t))
            .filter(|r| r.distance > self.distance)
    }
}

fn parse_races(s: &str) -> IResult<&str, Document> {
    separated_pair(
        preceded(tuple((tag("Time:"), space1)), separated_list1(space1, u32)),
        newline,
        preceded(
            tuple((tag("Distance:"), space1)),
            separated_list1(space1, u32),
        ),
    )
    .map(|(times, distances)| {
        izip!(times, distances)
            .map(|(time, distance)| Race { time, distance })
            .collect()
    })
    .map(Document)
    .parse(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case(Race::new(7, 9), &[(2,10), (3,12), (4,12), (5,10)])]
    fn sample_a_individual(#[case] race: Race, #[case] expectations: &[(u32, u32)]) {
        for (i, (expected, actual)) in expectations
            .iter()
            .map(|(t, d)| Race::new(*t, *d))
            .zip(race.winning_charge())
            .enumerate()
        {
            assert_eq!(expected, actual, "Race #{i}");
        }
    }

    #[test]
    fn sample_a() {
        let input = include_str!("../../sample/sixth.txt");
        let races = Document::from_str(input).expect("parsing");
        assert_eq!(288, races.margin());
    }
}
