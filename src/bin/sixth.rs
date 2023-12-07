use aoc23::Part;

use anyhow::anyhow;
use clap::Parser;
use itertools::izip;
use nom::{
    bytes::complete::tag,
    character::complete::{digit1, newline, space0, space1, u64},
    combinator::{map, peek},
    multi::{many_till, separated_list1},
    sequence::{preceded, separated_pair, terminated, tuple},
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
    let races = Document::parse(&input, args.part)?;
    let solution = races.margin();
    println!("Solution part {part:?}: {solution}", part = args.part);

    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct Race {
    time: u64,
    distance: u64,
}
impl Race {
    fn new(time: u64, distance: u64) -> Self {
        Self { time, distance }
    }
}

#[derive(Debug)]
struct Document(Vec<Race>);

impl Document {
    fn parse(s: &str, part: Part) -> anyhow::Result<Self> {
        let parser = match part {
            Part::One => parse_list_of_numbers,
            Part::Two => parse_single_number,
        };
        Ok(parse_races(s, parser)
            .finish()
            .map_err(|e| anyhow!("{e}"))?
            .1)
    }
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

fn parse_list_of_numbers(s: &str) -> IResult<&str, Vec<u64>> {
    separated_list1(space1, u64)(s)
}
fn parse_single_number(s: &str) -> IResult<&str, Vec<u64>> {
    map(
        many_till(terminated(digit1, space0), peek(newline)),
        |(digits, _)| vec![digits.join("").parse::<u64>().unwrap()],
    )(s)
}

fn parse_races<'a, P>(s: &'a str, numbers: P) -> IResult<&str, Document>
where
    P: NomParser<&'a str, Vec<u64>, nom::error::Error<&'a str>> + Clone,
{
    separated_pair(
        preceded(tuple((tag("Time:"), space1)), numbers.clone()),
        newline,
        preceded(tuple((tag("Distance:"), space1)), numbers),
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
    fn sample_a_individual(#[case] race: Race, #[case] expectations: &[(u64, u64)]) {
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
        let races = Document::parse(input, Part::One).expect("parsing");
        assert_eq!(288, races.margin());
    }

    #[test]
    fn sample_b() {
        let input = include_str!("../../sample/sixth.txt");
        let races = Document::parse(input, Part::Two).expect("parsing");
        assert_eq!(vec![Race::new(71530, 940200)], races.0);
        assert_eq!(71503, races.margin());
    }
}
