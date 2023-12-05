use std::{
    collections::{HashMap, HashSet, VecDeque},
    ops::Add,
    str::FromStr,
};

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

#[derive(Debug, Clone, Copy)]
struct Scratchcard {
    id: u32,
    wins: u32,
}

impl FromStr for Scratchcard {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(parse_card(s).finish().map_err(|e| anyhow!("{e}"))?.1)
    }
}

fn parse_card(s: &str) -> IResult<&str, Scratchcard> {
    let (s, (_, _, id, _, _)) = tuple((tag("Card"), space1, u32, tag(":"), space1))(s)?;
    let (s, winners) = separated_list1(space1, u32)
        .map(|list| HashSet::<u32>::from_iter(list.into_iter()))
        .parse(s)?;
    let (s, choices) = preceded(
        tuple((space1, tag("|"), space1)),
        separated_list1(space1, u32),
    )
    .map(|list| HashSet::from_iter(list.into_iter()))
    .parse(s)?;

    let wins = winners.intersection(&choices).count() as u32;
    Ok((s, Scratchcard { id, wins }))
}

fn main() -> anyhow::Result<()> {
    let args = Options::parse();

    let input = std::fs::read_to_string(&args.input)?;

    let solution = match args.part {
        Part::One => input
            .lines()
            .map(Scratchcard::from_str)
            .map_ok(|card| card.wins)
            .filter_ok(|wins| *wins > 0)
            .map_ok(|wins| 1 << (wins - 1))
            .fold_ok(0, Add::add)?,

        Part::Two => {
            let mut cards = HashMap::new();
            let originals = input
                .lines()
                .map(|line| Scratchcard::from_str(line).expect("Parsing ok"))
                .map(|card| (card.id, card))
                .collect::<HashMap<_, _>>();

            let mut queue = VecDeque::from_iter(originals.values());

            while let Some(card) = queue.pop_front() {
                cards
                    .entry(card.id)
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
                queue.extend(
                    ((card.id + 1)..=(card.id + card.wins)).filter_map(|id| originals.get(&id)),
                );
            }
            cards.values().sum()
        }
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
            .map(|line| Scratchcard::from_str(line).expect("Parsing ok"))
            .map(|card| card.wins)
            .collect::<Vec<_>>();
        assert_eq!(vec![4, 2, 2, 1, 0, 0], cards);
    }

    #[test]
    fn sample_b() {
        let input = include_str!("../../sample/fourth.txt");
        let mut cards = HashMap::new();
        let originals = input
            .lines()
            .map(|line| Scratchcard::from_str(line).expect("Parsing ok"))
            .map(|card| (card.id, card))
            .collect::<HashMap<_, _>>();

        let mut queue = VecDeque::from_iter(originals.values());

        while let Some(card) = queue.pop_front() {
            cards
                .entry(card.id)
                .and_modify(|count| *count += 1)
                .or_insert(1);
            queue.extend(
                ((card.id + 1)..=(card.id + card.wins)).map(|id| originals.get(&id).unwrap()),
            );
        }

        assert_eq!(Some(&1), cards.get(&1), "Card #1");
        assert_eq!(Some(&2), cards.get(&2), "Card #2");
        assert_eq!(Some(&4), cards.get(&3), "Card #3");
        assert_eq!(Some(&8), cards.get(&4), "Card #4");
        assert_eq!(Some(&14), cards.get(&5), "Card #5");
        assert_eq!(Some(&1), cards.get(&6), "Card #6");
    }
}
