#![feature(generators, iter_from_generator)]

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
    iter::once,
    ops::{BitAnd, Deref, Not},
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

    let mut springs = Springs::from_str(&input)?;
    for (i, mut report) in springs.0.iter_mut().enumerate() {
        println!("Report #{i}: {:?}", report.pattern());
        report.solve();
        println!("           : {:?}", report.pattern());
        println!("---------------------------------------------");
    }
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

#[derive(PartialEq, Eq, Debug, Default)]
struct Clue {
    len: u32,
    start: Option<usize>,
    end: Option<usize>,
}
impl Clue {
    fn new(len: u32) -> Self {
        Self {
            len,
            ..Default::default()
        }
    }
    fn solved(&self) -> bool {
        self.start.is_some() && self.end.is_some()
    }
}

#[derive(Debug, Default)]
struct Report {
    pattern: Pattern,
    clues: Vec<Clue>,
    solved_until: usize,
}
impl Report {
    fn pattern(&self) -> Pattern {
        Pattern(
            self.pattern
                .iter()
                .copied()
                .skip(1)
                .take(self.pattern.len() - 2)
                .collect(),
        )
    }
    fn clues(&self) -> impl Iterator<Item = &Clue> + DoubleEndedIterator + '_ {
        self.clues.iter()
    }
    fn clues_mut(&mut self) -> impl Iterator<Item = &mut Clue> + DoubleEndedIterator + '_ {
        self.clues.iter_mut()
    }
    fn clues_unsolved(&self) -> impl Iterator<Item = &Clue> + '_ {
        self.clues.iter().filter(|clue| clue.solved().not())
    }
    fn new(mut pattern: Pattern, groups: &[u32]) -> Self {
        pattern.0.insert(0, Bit::O);
        pattern.0.push(Bit::O);
        let groups = groups.iter().copied().map(Clue::new).collect();
        let solved_until = pattern.0.iter().take_while(|c| **c == Bit::O).count();
        Self {
            pattern,
            clues: groups,
            solved_until,
        }
    }
    fn combinations(&self) -> impl Iterator<Item = Combination> + '_ {
        let n = self.pattern.len();
        let m = self.clues.len() + 1;
        let k = n - self.clues().map(|clue| clue.len).sum::<u32>() as usize;
        (0..m)
            .map(|_| 1..k)
            .multi_cartesian_product()
            .filter(move |combi| k == combi.iter().sum::<usize>())
            .map(|combi| {
                combi
                    .into_iter()
                    .map(|i| (Bit::O, i))
                    .interleave(self.clues().map(|clue| (Bit::I, clue.len as usize)))
                    .flat_map(|(x, n)| repeat_n(x, n))
                    .collect::<Combination>()
            })
            .filter(|combi| {
                combi
                    .iter()
                    .zip(self.pattern.iter())
                    .all(|(a, b)| a.matches(&b))
            })
    }

    fn solve(&mut self) {
        while self
            .solve_simple_boxes()
            .and(self.solve_simple_spaces())
            .is_some()
        {
            println!("...");
        }
    }

    fn update_clue_bounds(&mut self) -> Option<()> {
        let mut clues = self.clues.iter_mut();
        let mut current_clue = clues.next()?;
        for (i, bit) in self.pattern.0.iter_mut().enumerate() {
            let len = current_clue.len as usize;
            if let Some(s) = current_clue.start {
                if i <= s {
                    // Before clue
                    continue;
                }

                if i <= s + len {
                    // Inside clue range
                    match bit {
                        Bit::I => continue,
                        Bit::X => {
                            println!("Found X at #{i} while processing clue {current_clue:?}, so setting it to I");
                            *bit = Bit::I;
                        }
                        Bit::O => panic!(
                            "Contradiction: Expected a group from {s}..{} but found a O at {i}",
                            s + len
                        ),
                    }
                    continue;
                }
                if i == s + len + 1 {
                    // End of clue
                    match bit {
                        Bit::I => panic!("Contradiction: Expected a group to end at #{}, thus a space at #{i} but found an I here", i-1),
                        Bit::X => {
                            println!("Found X at #{i} while closing clue {current_clue:?}, so setting it to O");
                            *bit = Bit::O;
                        },
                        Bit::O => {},
                    }
                    if current_clue.end.is_none() {
                        current_clue.end = Some(i);
                    }
                    current_clue = clues.next()?;
                }
            } else {
                // No start yet
                if *bit == Bit::X {
                    println!("Cannot reliably determine clue start because it might be inside an X block");
                    return None;
                }
                if *bit == Bit::I {
                    current_clue.start = Some(i - 1);
                    println!("Staring clue {current_clue:?}");
                }
            }
        }
        None
    }

    fn solve_simple_boxes(&mut self) -> Option<()> {
        dbg!(&self.pattern);
        let n = self
            .clues()
            .map(|clue| clue.len)
            .intersperse(1)
            .sum::<u32>() as usize;
        dbg!(n);
        let fill = self.pattern.len().saturating_sub(n + 2);
        dbg!(fill);
        let upper = repeat_n(Bit::X, fill)
            .chain(once(Bit::O))
            .chain(
                self.clues()
                    .map(|clue| repeat_n(Bit::I, clue.len as usize))
                    .intersperse(repeat_n(Bit::O, 1))
                    .flatten(),
            )
            .chain(once(Bit::O))
            .collect::<Combination>();

        let lower = once(Bit::O)
            .chain(
                self.clues()
                    .map(|clue| repeat_n(Bit::I, clue.len as usize))
                    .intersperse(repeat_n(Bit::O, 1))
                    .flatten(),
            )
            .chain(once(Bit::O))
            .chain(repeat_n(Bit::X, fill))
            .collect::<Combination>();

        dbg!(&lower, &upper);
        let intersection = lower & upper;

        if intersection.iter().all(|bit| *bit != Bit::X) && self.pattern != intersection {
            self.pattern = intersection;
            return Some(());
        }

        dbg!(&intersection);
        let mut changed = None;
        for ((i, bit), _) in self
            .pattern
            .0
            .iter_mut()
            .enumerate()
            .zip(intersection.iter())
            .filter(|((i, a), b)| **b == Bit::I && **a != Bit::I)
        {
            println!("Setting bit #{i} to I");
            // *bit = Bit::I;
            changed = Some(());
        }
        dbg!(&self.pattern);
        changed
    }

    fn solve_simple_spaces(&mut self) -> Option<()> {
        self.update_clue_bounds()?;
        None
    }

    // fn solve_once(&mut self) -> Option<()> {
    //     let mut unsolved = self.clues_unsolved();
    //     let clue = unsolved.next()?;
    //     let mut iter = self.pattern.iter().skip(self.solved_until - 1).multipeek();
    //     let g = clue.len as usize;
    //
    //     if let Some(remaining) = intersperse(unsolved.map(|clue| clue.len), 1).sum1::<u32>() {
    //         let upper_bound_last_zero = self.pattern.len() - 2 - remaining as usize;
    //         let lower_bound_first_zero = self.solved_until + g;
    //
    //         // dbg!(lower_bound_first_zero, upper_bound_last_zero);
    //         if lower_bound_first_zero == upper_bound_last_zero
    //             && self.pattern.0[lower_bound_first_zero] == Bit::X
    //         {
    //             println!("Found case where groups match exactly, pattern@{lower_bound_first_zero} must be zero");
    //             self.pattern.0[lower_bound_first_zero] = Bit::O;
    //             return Some(());
    //         }
    //     }
    //
    //     loop {
    //         if (0..g + 2)
    //             .filter_map(|_| iter.peek().copied())
    //             .any(|c| *c != Bit::X)
    //         {
    //             break;
    //         }
    //         if iter.next().is_none() {
    //             break;
    //         }
    //     }
    //
    //     let window = Pattern(iter.take(g + 2).copied().collect::<Vec<_>>());
    //     let mut solutions = window
    //         .combinations()
    //         // .inspect(|combi| {
    //         //     dbg!(combi);
    //         // })
    //         .filter(|combi| {
    //             g == combi
    //                 .iter()
    //                 .skip_while(|c| *c == Bit::O)
    //                 .take_while(|c| *c == Bit::I)
    //                 .count()
    //         });
    //     let solution = solutions.next()?;
    //     if solutions.next().is_some() {
    //         // more than one solution found, cannot solve
    //         // dbg!("More than one solution found");
    //         return None;
    //     }
    //
    //     // Solution found for group g
    //     let new_solved_until = self.solved_until + g + 1;
    //     let group = self
    //         .clues
    //         .iter_mut()
    //         .skip_while(|clue| clue.solved())
    //         .next()?;
    //
    //     group.start = Some(self.solved_until);
    //     group.end = Some(new_solved_until);
    //
    //     // dbg!(g, self.solved_until, new_solved_until, &self.pattern);
    //     self.pattern.0 = self.pattern.0[..self.solved_until]
    //         .into_iter()
    //         .copied()
    //         .chain(solution.iter().skip(1))
    //         .chain(self.pattern.0[new_solved_until..].into_iter().copied())
    //         .collect_vec();
    //
    //     self.solved_until = new_solved_until;
    //     // dbg!(&window, solution, &self.pattern);
    //     Some(())
    // }
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
impl Bit {
    fn matches(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::X, _) => true,
            (_, Self::X) => true,
            (a, b) => a == b,
        }
    }
}
impl BitAnd for Bit {
    type Output = Self;
    fn bitand(self, other: Self) -> Self::Output {
        match (self, other) {
            (Bit::I, Bit::I) => Bit::I,
            (Bit::O, Bit::O) => Bit::O,
            _ => Bit::X,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Hash)]
struct Combination {
    pattern: u32,
    len: usize,
}
impl Debug for Combination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for bit in 0..self.len {
            write!(
                f,
                "{}",
                if self.pattern & (1 << bit) > 0 {
                    Bit::I
                } else {
                    Bit::O
                }
            )?;
        }
        Ok(())
    }
}
impl FromStr for Combination {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(combination(s).finish().map_err(anyhowing)?.1)
    }
}
impl FromIterator<Bit> for Combination {
    fn from_iter<T: IntoIterator<Item = Bit>>(iter: T) -> Self {
        let mut len = 0;
        let mut pattern = 0u32;
        let mut iter = iter.into_iter();
        while let Some(bit) = iter.next() {
            if bit == Bit::I {
                pattern |= 1 << len;
            }
            len += 1;
        }
        Self { pattern, len }
    }
}

impl IntoIterator for Combination {
    type Item = Bit;
    type IntoIter = CombinationIter;

    fn into_iter(self) -> Self::IntoIter {
        CombinationIter {
            index: 0,
            pattern: self.pattern,
            len: self.len,
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
struct CombinationIter {
    index: usize,
    pattern: u32,
    len: usize,
}
impl Iterator for CombinationIter {
    type Item = Bit;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            return None;
        }

        let c = self.pattern & (1 << self.index) > 0;
        self.index += 1;
        Some(c.into())
    }
}

impl Combination {
    fn iter(&self) -> impl Iterator<Item = Bit> + Clone + '_ {
        (0..self.len)
            .map(|bit| self.pattern & (1 << bit) > 0)
            .map(Bit::from)
    }
}

impl PartialEq<Pattern> for Combination {
    fn eq(&self, other: &Pattern) -> bool {
        self.iter().zip(other.0.iter()).all(|(a, b)| a.matches(b))
    }
}
impl BitAnd for Combination {
    type Output = Pattern;
    fn bitand(self, other: Self) -> Self::Output {
        Pattern(self.iter().zip(other.iter()).map(|(a, b)| a & b).collect())
    }
}

#[derive(Default, PartialEq, Eq, Clone, Hash)]
struct Pattern(Vec<Bit>);

impl Pattern {
    fn combinations(&self) -> impl Iterator<Item = Combination> + '_ {
        self.iter()
            .map(|c| match c {
                Bit::I => Combination::from_iter(once(Bit::I)),
                Bit::O => Combination::from_iter(once(Bit::O)),
                Bit::X => Combination::from_iter([Bit::I, Bit::O].into_iter()),
            })
            .multi_cartesian_product()
            .map(|iter| Combination::from_iter(iter))
    }
}
impl Deref for Pattern {
    type Target = [Bit];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Debug for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().map(|p| p.to_string()).join(""),)
    }
}
impl FromStr for Pattern {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(pattern(s).finish().map_err(anyhowing)?.1)
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
impl From<bool> for Bit {
    fn from(value: bool) -> Self {
        if value {
            Self::I
        } else {
            Self::O
        }
    }
}

#[derive(Debug, Default)]
struct Springs(Vec<Report>);
impl Springs {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn reports(&self) -> impl Iterator<Item = &Report> {
        self.0.iter()
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
fn combination(s: &str) -> IResult<&str, Combination> {
    many1(alt((char('.').value(Bit::O), char('#').value(Bit::I))))
        .map(|cs| cs.into_iter().collect())
        .parse(s)
}
fn report(s: &str) -> IResult<&str, Report> {
    pattern
        .terminated(space1)
        .and(separated_list1(char(','), u32))
        .map(|(pattern, groups)| Report::new(pattern, groups.as_slice()))
        .parse(s)
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
            .map(|x| combination(&format!(".{x}.")).unwrap().1)
        {
            assert!(
                combinations.contains(&expected),
                "\nPattern  {:?}\nExpected {expected:?}",
                report.pattern
            );
        }
    }

    #[rstest]
    #[case(". 0", ".")]
    #[case("# 1", "#")]
    #[case("? 1", "#")]
    #[case("?? 1", "??")]
    #[case("#? 2", "##")]
    #[case("?# 2", "##")]
    #[case("#?. 2", "##.")]
    #[case(".#? 2", ".##")]
    #[case("#?? 2", "##.")]
    #[case(".##. 2", ".##.")]
    #[case(".????? 2", ".?????")]
    #[case("??# 1,1", "#.#")]
    #[case("#?# 1,1", "#.#")]
    #[case("?.? 1,1", "#.#")]
    #[case("#?#? 1,1", "#.#.")]
    #[case("???.### 1,1,3", "#.#.###")]
    #[case("#?#?#?#?#?#? 1,3,6", "#.###.######")]
    #[case("#?#?#?#?#?#?#? 1,3,1,6", "#.###.#.######")]
    #[case("????.######..#####. 1,6,5", "????.######..#####?")]
    // #[case("?###???????? 3,2,1", ".###.???????")]
    fn solve(#[case] mut report: Report, #[case] expected_pattern: Pattern) {
        report.solve();
        assert_eq!(expected_pattern, report.pattern());
    }

    #[rstest]
    #[case("...", vec!["..."])]
    #[case(".", vec!["."])]
    #[case("#", vec!["#"])]
    #[case("?", vec![".","#"])]
    #[case(".??.", vec![
           "....",
           "..#.",
           ".#..",
           ".##.",
    ])]
    fn pattern_combinations(#[case] pattern: Pattern, #[case] expected_combinations: Vec<&str>) {
        let combinations = pattern.combinations().collect::<HashSet<_>>();
        assert_eq!(expected_combinations.len(), combinations.len());

        for expected in expected_combinations
            .into_iter()
            .map(|s| Combination::from_str(s).expect("valid combination"))
        {
            assert!(
                combinations.contains(&expected),
                "\n Pattern  {pattern:?}\nExpected {expected:?}"
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
