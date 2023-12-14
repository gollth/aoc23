#![feature(generators, iter_from_generator)]

use aoc23::{anyhowing, Part};

use anyhow::Result;
use clap::Parser;
use itertools::Itertools;
use nom::{
    branch::alt,
    character::complete::{char, space1, u32},
    multi::{many1, separated_list1},
    Finish, IResult, Parser as NomParser,
};
use nom_supreme::ParserExt;
use std::{
    collections::{HashMap, VecDeque},
    fmt::{Debug, Display},
    iter::repeat,
    str::FromStr,
};

/// Day 12: Hot Springs
#[derive(Debug, Parser)]
struct Options {
    /// Path to the file with the input data
    #[clap(short, long, default_value = "sample/twelfth.txt")]
    input: String,

    /// Which part of the day to solve
    part: Part,
}

fn main() -> anyhow::Result<()> {
    let args = Options::parse();
    let input = std::fs::read_to_string(&args.input)?;

    let input = match args.part {
        Part::One => input,
        Part::Two => input
            .lines()
            .flat_map(|line| line.split_whitespace().collect_tuple())
            .map(|(pattern, clues)| {
                format!(
                    "{} {}",
                    repeat(pattern).take(5).join("?"),
                    repeat(clues).take(5).join(","),
                )
            })
            .join("\n"),
    };

    let springs = Springs::from_str(&input)?;
    let solution = springs
        .reports()
        .map(|report| report.arrangements())
        .sum::<usize>();

    println!("Solution part {part:?}: {solution}", part = args.part);
    Ok(())
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
enum Clue {
    Unknown(u32),
    Checking(u32),
}

type Memo = HashMap<(Option<Bit>, Option<Clue>, VecDeque<Bit>, VecDeque<Clue>), usize>;

fn recurse(
    memo: &mut Memo,
    bit: Option<Bit>,
    clue: Option<Clue>,
    mut bits: VecDeque<Bit>,
    mut clues: VecDeque<Clue>,
) -> usize {
    let key = (bit, clue, bits.clone(), clues.clone());
    if let Some(cache) = memo.get(&key) {
        return *cache;
    }

    let result = match (bit, clue) {
        // all clues and all bits consumed, this is a valid solution
        (None, None) => 1,

        // not all clues yet consumed, this is not a valid solution
        (None, Some(_)) => 0,

        // no clue left but another I found, this is not a valid solution
        (Some(Bit::I), None) => 0,

        // found a padding zero bit, remove it and recurse
        (Some(Bit::O), None) => recurse(memo, bits.pop_front(), clue, bits, clues),

        // No active clue right now, but a O doesnt start one yet, just recurse
        (Some(Bit::O), Some(Clue::Unknown(_))) => {
            recurse(memo, bits.pop_front(), clue, bits, clues)
        }

        // No active clue right now, but this I starts the next, recurse with next clue
        (Some(Bit::I), Some(Clue::Unknown(l))) => {
            recurse(memo, bit, Some(Clue::Checking(l)), bits, clues)
        }

        // end of a clue
        (Some(Bit::O), Some(Clue::Checking(0))) => {
            recurse(memo, bits.pop_front(), clues.pop_front(), bits, clues)
        }

        // Found O while expected a block of at least n Is, thus invalid solution
        (Some(Bit::O), Some(Clue::Checking(_n))) => 0,

        // expand the X with both I + O and recurse
        (Some(Bit::X), _) => {
            recurse(memo, Some(Bit::I), clue, bits.clone(), clues.clone())
                + recurse(memo, Some(Bit::O), clue, bits, clues)
        }

        // clue does not indicate more Is to come, but we found another, thus invalid solution
        (Some(Bit::I), Some(Clue::Checking(0))) => 0,

        // checking a block of Is against a clue, recurse
        (Some(Bit::I), Some(Clue::Checking(l))) => recurse(
            memo,
            bits.pop_front(),
            Some(Clue::Checking(l - 1)),
            bits,
            clues,
        ),
    };

    memo.insert(key, result);
    result
}

#[derive(Debug, Default)]
struct Report {
    pattern: Pattern,
    clues: Vec<u32>,
}
impl Report {
    fn new(mut pattern: Pattern, clues: Vec<u32>) -> Self {
        pattern.0.push(Bit::O);
        Self { pattern, clues }
    }

    fn arrangements(&self) -> usize {
        let mut bits = self.pattern.0.iter().copied().collect::<VecDeque<_>>();
        let mut clues = self
            .clues
            .iter()
            .map(|n| Clue::Unknown(*n))
            .collect::<VecDeque<_>>();

        let mut memo = HashMap::new();
        recurse(&mut memo, bits.pop_front(), clues.pop_front(), bits, clues)
    }
}
impl FromStr for Report {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(report(s).finish().map_err(anyhowing)?.1)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
enum Bit {
    I,
    O,
    X,
}

#[derive(Default, PartialEq, Eq, Clone, Hash)]
struct Pattern(Vec<Bit>);
impl Debug for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().map(|p| p.to_string()).join(""),)
    }
}

impl Display for Bit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Bit::I => write!(f, "█"),
            Bit::O => write!(f, "·"),
            Bit::X => write!(f, "░"),
        }
    }
}

#[derive(Debug, Default)]
struct Springs(Vec<Report>);
impl Springs {
    fn reports(&self) -> impl Iterator<Item = &Report> {
        self.0.iter()
    }
}
impl FromStr for Springs {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Springs(
            s.lines()
                .map(Report::from_str)
                .collect::<Result<Vec<_>>>()?,
        ))
    }
}

fn condition(s: &str) -> IResult<&str, Bit> {
    alt((
        char('.').value(Bit::O),
        char('#').value(Bit::I),
        char('?').value(Bit::X),
    ))
    .parse(s)
}

fn pattern(s: &str) -> IResult<&str, Pattern> {
    many1(condition).map(Pattern).parse(s)
}
fn report(s: &str) -> IResult<&str, Report> {
    pattern
        .terminated(space1)
        .and(separated_list1(char(','), u32))
        .map(|(pattern, groups)| Report::new(pattern, groups))
        .parse(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case("# 1", 1)]
    #[case("## 1", 0)] // invalid
    #[case(".# 1", 1)]
    #[case(".....# 1", 1)]
    #[case("? 1", 1)]
    #[case("?? 1", 2)]
    #[case("??? 1,1", 1)]
    #[case("#.#..### 1,1,3", 1)]
    #[case("???.### 1,1,3", 1)]
    #[case(".??..??...?##. 1,1,3", 4)]
    #[case("?#?#?#?#?#?#?#? 1,3,1,6", 1)]
    #[case("????.#...#... 4,1,1", 1)]
    #[case("????.######..#####. 1,6,5", 4)]
    #[case("#???? 1,2", 2)]
    #[case("?###???????? 3,2,1", 10)]
    fn sample_a_manual(#[case] report: Report, #[case] expected_combinations: usize) {
        assert_eq!(expected_combinations, report.arrangements());
    }

    #[rstest]
    fn sample_a() {
        let input = include_str!("../../sample/twelfth.txt");
        let springs = Springs::from_str(input).expect("parsing");
        let arrangements = springs
            .reports()
            .map(|report| report.arrangements())
            .sum::<usize>();
        assert_eq!(21, arrangements);
    }
}
