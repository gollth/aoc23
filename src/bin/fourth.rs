use std::{collections::HashSet, ops::Add, str::FromStr};

use anyhow::anyhow;
use aoc23::Part;
use clap::Parser;
use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::{space1, u32},
    multi::separated_list1,
    sequence::{preceded, tuple},
    Finish, IResult, Parser as NomParser,
};

/// Day 4: Scratchcards
#[derive(Parser)]
struct Options {
    /// Path to the file with the input data
    #[clap(short, long, default_value = "sample/fourth.txt")]
    input: String,

    /// Which part of the day to solve
    part: Part,
}

#[derive(Debug)]
struct Scratchcard {
    winners: HashSet<u32>,
    choices: HashSet<u32>,
}

impl Scratchcard {
    fn wins(&self) -> usize {
        self.winners.intersection(&self.choices).count()
    }
}

impl FromStr for Scratchcard {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(parse_card(s).finish().map_err(|e| anyhow!("{e}"))?.1)
    }
}

fn parse_card(s: &str) -> IResult<&str, Scratchcard> {
    let card = tuple((tag("Card"), space1, u32, tag(":"), space1));
    let (s, winners) = preceded(card, separated_list1(space1, u32))
        .map(|list| HashSet::from_iter(list.into_iter()))
        .parse(s)?;
    let (s, choices) = preceded(
        tuple((space1, tag("|"), space1)),
        separated_list1(space1, u32),
    )
    .map(|list| HashSet::from_iter(list.into_iter()))
    .parse(s)?;
    Ok((s, Scratchcard { winners, choices }))
}

fn main() -> anyhow::Result<()> {
    let args = Options::parse();

    let input = std::fs::read_to_string(&args.input)?;

    let solution = match args.part {
        Part::One => input
            .lines()
            .map(Scratchcard::from_str)
            .map_ok(|card| card.wins())
            .filter_ok(|wins| *wins > 0)
            .map_ok(|wins| 1 << (wins - 1))
            .fold_ok(0, Add::add)?,
        Part::Two => unimplemented!(),
    };
    println!("Solution part {part:?}: {solution}", part = args.part);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_a() {
        let input = include_str!("../../sample/fourth.txt");
        let cards = input
            .lines()
            .filter_map(|line| Scratchcard::from_str(line).ok())
            .map(|card| card.wins())
            .collect::<Vec<_>>();
        assert_eq!(vec![4, 2, 2, 1, 0, 0], cards);
    }
}
