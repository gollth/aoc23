use std::{collections::HashMap, fmt::Debug, iter::once, ops::Range, str::FromStr};

use aoc23::Part;

use anyhow::{anyhow, Result};
use clap::Parser;
use enum_iterator::{all, Sequence};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, i128, line_ending, space1},
    combinator::map,
    multi::{many_till, separated_list1},
    sequence::{preceded, separated_pair, terminated, tuple},
    Finish, IResult, Parser as NomParser,
};

/// Day 5: If You Give A Seed A Fertilizer
#[derive(Debug, Parser)]
struct Options {
    /// Path to the file with the input data
    #[clap(short, long, default_value = "sample/fifth.txt")]
    input: String,

    /// Which part of the day to solve
    part: Part,
}

#[derive(PartialEq, Eq, Clone)]
struct Mapping {
    range: Range<i128>,
    offset: i128,
}

impl Mapping {
    fn new(range: Range<i128>, offset: i128) -> Self {
        Self { range, offset }
    }

    fn len(&self) -> i128 {
        self.range.end - self.range.start
    }
}

impl Debug for Mapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}..{} -> {}",
            self.range.start,
            self.range.end - 1,
            self.offset,
        )
    }
}

fn propagate(rs: &[Range<i128>], ts: &[Mapping]) -> Vec<Range<i128>> {
    let mut ranges = rs.to_vec();
    ts.iter()
        .chain(once(&Mapping::new(0..i128::MAX, 0)))
        .flat_map(|t| {
            let mut news = Vec::new();
            let mut olds = Vec::new();
            for range in &ranges {
                if range.end <= t.range.start {
                    // other range is entirely to the right of us
                    olds.push(range.clone());
                    continue;
                }
                if t.range.end <= range.start {
                    // other range is entirely to the left of us
                    olds.push(range.clone());
                    continue;
                }

                if t.range.start < range.start && t.range.end < range.end {
                    // other range starts left from us and stops inside our range
                    olds.push(t.range.end..range.end);
                    news.push(range.start + t.offset..t.range.end + t.offset);
                    continue;
                }
                if range.start < t.range.start && range.end < t.range.end {
                    // other range starts inside our and stops outside our range
                    olds.push(range.start..t.range.start);
                    news.push(t.range.start + t.offset..range.end + t.offset);
                    continue;
                }
                if (range.end - range.start) < t.len() {
                    // other range covers entirely our range
                    news.push(range.start + t.offset..range.end + t.offset);
                    continue;
                }

                // other range is entirely inside our range
                olds.push(range.start..t.range.start);
                olds.push(t.range.end..range.end);
                news.push(t.range.start + t.offset..t.range.end + t.offset);
            }
            ranges = olds;
            news
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sequence)]
enum Resource {
    Soil,
    Fertilizer,
    Water,
    Light,
    Temperature,
    Humidity,
    Location,
}

#[derive(Debug)]
struct Almanac(HashMap<Resource, Vec<Mapping>>);

fn parse_seeds_individual(s: &str) -> IResult<&str, Vec<Range<i128>>> {
    separated_list1(space1, map(i128, |x| x..(x + 1)))(s)
}

fn parse_seeds_ranges(s: &str) -> IResult<&str, Vec<Range<i128>>> {
    separated_list1(
        space1,
        map(separated_pair(i128, space1, i128), |(a, b)| a..(a + b)),
    )(s)
}

impl Almanac {
    fn parse(part: Part, s: &str) -> Result<(Self, Vec<Range<i128>>)> {
        let parser = match part {
            Part::One => parse_seeds_individual,
            Part::Two => parse_seeds_ranges,
        };
        let (s, seeds) = preceded(tag("seeds: "), parser)(s).map_err(|e| anyhow!("{e}"))?;
        let almanac = Self::from_str(s)?;
        Ok((almanac, seeds))
    }

    fn best_location(&self, seeds: &[Range<i128>]) -> i128 {
        all::<Resource>()
            .fold(seeds.to_vec(), |ranges, resource| {
                // dbg!(&resource, &ranges);
                let mappings = self
                    .0
                    .get(&resource)
                    .unwrap_or_else(|| panic!("Almanac to contain mapping to {resource:?}"));
                propagate(&ranges, mappings)
            })
            .iter()
            .map(|r| r.start)
            .min()
            .expect("Seeds not to be empty")
    }
}

impl FromStr for Almanac {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(parse_almanac(s).finish().map_err(|e| anyhow!("{e}"))?.1)
    }
}
fn parse_mapping(s: &str) -> IResult<&str, Mapping> {
    tuple((terminated(i128, space1), terminated(i128, space1), i128))
        .map(|(dest, src, len)| Mapping::new(src..(src + len), dest - src))
        .parse(s)
}

fn parse_header(s: &str) -> IResult<&str, Resource> {
    preceded(
        many_till(anychar, tag("-to-")),
        terminated(parse_resource, tuple((tag(" map:"), line_ending))),
    )(s)
}

fn parse_almanac(s: &str) -> IResult<&str, Almanac> {
    separated_list1(
        tuple((line_ending, line_ending)),
        tuple((parse_header, separated_list1(line_ending, parse_mapping))),
    )
    .map(|items| items.into_iter().collect())
    .map(Almanac)
    .parse(s)
}

fn parse_resource(s: &str) -> IResult<&str, Resource> {
    alt((
        map(tag("soil"), |_| Resource::Soil),
        map(tag("fertilizer"), |_| Resource::Fertilizer),
        map(tag("water"), |_| Resource::Water),
        map(tag("light"), |_| Resource::Light),
        map(tag("temperature"), |_| Resource::Temperature),
        map(tag("humidity"), |_| Resource::Humidity),
        map(tag("location"), |_| Resource::Location),
    ))(s)
}

fn main() -> Result<()> {
    let args = Options::parse();
    let input = std::fs::read_to_string(args.input)?;
    let (almanac, seeds) = Almanac::parse(args.part, &input)?;
    let solution = almanac.best_location(&seeds);
    println!("Solution part {:?}: {solution}", args.part);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(79, 82)]
    #[case(14, 43)]
    #[case(55, 86)]
    #[case(13, 35)]
    fn sample_a(#[case] seed: i128, #[case] location: i128) {
        let input = include_str!("../../sample/fifth.txt");
        let (almanac, seeds) = Almanac::parse(Part::One, input).unwrap();
        let seed = seed..(seed + 1);
        assert!(seeds.contains(&seed));
        assert_eq!(location, almanac.best_location(&[seed]));
    }

    #[rstest]
    #[case(79..(79+14), 46)]
    #[case(55..(55+13), 56)]
    fn sample_b(#[case] seed: Range<i128>, #[case] location: i128) {
        let input = include_str!("../../sample/fifth.txt");
        let (almanac, _) = Almanac::parse(Part::Two, input).unwrap();
        assert_eq!(location, almanac.best_location(&[seed]));
    }

    #[test]
    fn sample_b_manual() {
        let x = vec![55..68, 79..93];
        // Seed -> Soil
        let x = propagate(&x, &[Mapping::new(98..100, -48), Mapping::new(50..98, 2)]);

        // Soil -> Fertilizer
        let x = propagate(
            &x,
            &[
                Mapping::new(98..100, -48),
                Mapping::new(52..54, -15),
                Mapping::new(0..15, 39),
            ],
        );

        // Fertilizer -> Water
        let x = propagate(
            &x,
            &[
                Mapping::new(53..61, -4),
                Mapping::new(11..53, -11),
                Mapping::new(0..7, 42),
                Mapping::new(7..11, 50),
            ],
        );

        // Water -> Light
        let x = propagate(&x, &[Mapping::new(18..25, 70), Mapping::new(25..95, -7)]);

        // Light -> Temperature
        let x = propagate(
            &x,
            &[
                Mapping::new(77..100, -32),
                Mapping::new(45..64, 36),
                Mapping::new(64..77, 4),
            ],
        );
        // Temperature -> Humidity
        let x = propagate(&x, &[Mapping::new(69..70, -69), Mapping::new(0..69, 1)]);

        // Humidity -> Location
        let x = propagate(&x, &[Mapping::new(56..93, 4), Mapping::new(93..97, -37)]);

        let mut x = x;
        x.sort_by_key(|r| r.start);

        assert_eq!(46, x[0].start);
    }
}
