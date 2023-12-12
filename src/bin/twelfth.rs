use aoc23::{anyhowing, Part};

use anyhow::Result;
use clap::Parser;
use indicatif::ProgressIterator;
use itertools::{repeat_n, Itertools};
use nom::{
    branch::alt,
    character::complete::{char, space1, u32},
    multi::{many1, separated_list1},
    Finish, IResult, Parser as NomParser,
};
use nom_supreme::ParserExt;
use std::{
    fmt::{Debug, Display},
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

    let springs = Springs::from_str(&input)?;
    let solution = match args.part {
        Part::One => springs
            .reports()
            .map(|report| report.combinations().count())
            .progress_count(springs.len() as u64)
            .sum::<usize>(),
        Part::Two => unimplemented!(),
    };

    println!("Solution part {part:?}: {solution}", part = args.part);
    Ok(())
}

#[derive(Debug, Default)]
struct Springs(Vec<Report>);

#[derive(Debug, Default)]
struct Report {
    pattern: Pattern,
    groups: Vec<u32>,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
enum Condition {
    I,
    O,
    X,
}

impl Condition {
    fn matches(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::X, _) => true,
            (_, Self::X) => true,
            (a, b) => a == b,
        }
    }
}

#[derive(Default, PartialEq, Eq, Clone, Hash)]
struct Pattern(Vec<Condition>);

impl Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Condition::I => write!(f, "█"),
            Condition::O => write!(f, "·"),
            Condition::X => write!(f, "░"),
        }
    }
}
impl Debug for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().map(|p| p.to_string()).join(""),)
    }
}
impl FromStr for Springs {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Springs(
            s.lines()
                .map(|line| Report::from_str(line))
                .collect::<Result<Vec<_>>>()?,
        ))
    }
}

impl FromStr for Report {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(report(s).finish().map_err(anyhowing)?.1)
    }
}

fn condition(s: &str) -> IResult<&str, Condition> {
    alt((
        char('.').value(Condition::O),
        char('#').value(Condition::I),
        char('?').value(Condition::X),
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

impl Report {
    fn new(mut pattern: Pattern, groups: Vec<u32>) -> Self {
        pattern.0.insert(0, Condition::O);
        pattern.0.push(Condition::O);
        Self { pattern, groups }
    }
    fn combinations(&self) -> impl Iterator<Item = Pattern> + '_ {
        let n = self.pattern.0.len();
        let m = self.groups.len() + 1;
        let k = n - self.groups.iter().sum::<u32>() as usize;
        (0..m)
            .map(|_| 1..k)
            .multi_cartesian_product()
            .filter(move |combi| k == combi.iter().sum::<usize>())
            .map(|combi| {
                combi
                    .into_iter()
                    .map(|i| (Condition::O, i))
                    .interleave(self.groups.iter().map(|x| (Condition::I, *x as usize)))
                    .flat_map(|(x, n)| repeat_n(x, n))
                    .collect::<Vec<_>>()
            })
            .filter(|combi| {
                combi
                    .iter()
                    .zip(self.pattern.0.iter())
                    .all(|(a, b)| a.matches(&b))
            })
            .map(Pattern)
    }
}

impl Springs {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn reports(&self) -> impl Iterator<Item = &Report> {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case("#.#..### 1,1,3", vec![
           "#.#..###"
    ])]
    #[case("???.### 1,1,3", vec![
           "#.#.###"
    ])]
    #[case(".??..??...?##. 1,1,3", vec![
           ".#...#....###.",
           "..#..#....###.",
           ".#....#...###.",
           "..#...#...###.",
    ])]
    #[case("?#?#?#?#?#?#?#? 1,3,1,6", vec![
           ".#.###.#.######"
    ])]
    #[case("????.#...#... 4,1,1", vec![
           "####.#...#..."
    ])]
    #[case("????.######..#####. 1,6,5", vec![
           "#....######..#####.",
           ".#...######..#####.",
           "..#..######..#####.",
           "...#.######..#####.",
    ])]
    #[case("?###???????? 3,2,1", vec![
           ".###.##.#...",
           ".###.##..#..",
           ".###.##...#.",
           ".###.##....#",
           ".###..##.#..",
           ".###..##..#.",
           ".###..##...#",
           ".###...##.#.",
           ".###...##..#",
           ".###....##.#",
    ])]
    fn sample_a_manual(#[case] report: Report, #[case] expected_combinations: Vec<&str>) {
        let combinations = report.combinations().collect::<HashSet<_>>();
        assert_eq!(expected_combinations.len(), combinations.len());

        for expected in expected_combinations
            .into_iter()
            .map(|x| pattern(&format!(".{x}.")).unwrap().1)
        {
            assert!(
                combinations.contains(&expected),
                "\nPattern  {:?}\nExpected {expected:?}",
                report.pattern
            );
        }
    }

    #[rstest]
    fn sample_a() {
        let input = include_str!("../../sample/twelfth.txt");
        let springs = Springs::from_str(input).expect("parsing");
        let arrangements = springs
            .reports()
            .map(|report| report.combinations().count())
            .sum::<usize>();
        assert_eq!(21, arrangements);
    }
}
