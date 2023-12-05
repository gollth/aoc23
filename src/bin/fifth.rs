use std::{collections::HashMap, str::FromStr};

use aoc23::Part;

use anyhow::{anyhow, Result};
use clap::Parser;
use enum_iterator::{all, Sequence};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, line_ending, space1, u128},
    combinator::map,
    multi::{many_till, separated_list1},
    sequence::{preceded, terminated, tuple},
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

#[derive(Debug)]
struct Range {
    src: u128,
    dest: u128,
    len: u128,
}

#[derive(Debug)]
struct LookupTable(Vec<Range>);

impl LookupTable {
    fn lookup(&self, idx: u128) -> u128 {
        self.0
            .iter()
            .find(|r| r.src <= idx && idx < r.src + r.len)
            .map(|r| idx - r.src + r.dest)
            .unwrap_or(idx)
    }
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
struct Almanac {
    seeds: Vec<u128>,
    pages: HashMap<Resource, LookupTable>,
}

impl Almanac {
    fn location(&self, seed: u128) -> u128 {
        all::<Resource>().fold(seed, |i, resource| self.lookup(i, resource))
    }

    fn lookup(&self, idx: u128, resource: Resource) -> u128 {
        self.pages
            .get(&resource)
            .unwrap_or_else(|| panic!("Almanac to contain mapping to {resource:?}"))
            .lookup(idx)
    }
}

impl FromStr for Almanac {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(parse_almanac(s).finish().map_err(|e| anyhow!("{e}"))?.1)
    }
}
fn parse_range(s: &str) -> IResult<&str, Range> {
    tuple((terminated(u128, space1), terminated(u128, space1), u128))
        .map(|(dest, src, len)| Range { src, dest, len })
        .parse(s)
}

fn parse_header(s: &str) -> IResult<&str, Resource> {
    preceded(
        many_till(anychar, tag("-to-")),
        terminated(parse_resource, tuple((tag(" map:"), line_ending))),
    )(s)
}

fn parse_lookup_table(s: &str) -> IResult<&str, LookupTable> {
    separated_list1(line_ending, parse_range)
        .map(LookupTable)
        .parse(s)
}

fn parse_pages(s: &str) -> IResult<&str, HashMap<Resource, LookupTable>> {
    separated_list1(
        tuple((line_ending, line_ending)),
        tuple((parse_header, parse_lookup_table)),
    )
    .map(|items| items.into_iter().collect())
    .parse(s)
}

fn parse_almanac(s: &str) -> IResult<&str, Almanac> {
    tuple((
        preceded(tag("seeds: "), separated_list1(space1, u128)),
        parse_pages,
    ))
    .map(|(seeds, pages)| Almanac { seeds, pages })
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
    let solution = match args.part {
        Part::One => {
            let input = std::fs::read_to_string(args.input)?;
            let almanac = Almanac::from_str(&input)?;
            almanac
                .seeds
                .iter()
                .map(|seed| almanac.location(*seed))
                .min()
                .expect("No min location found")
        }
        Part::Two => unimplemented!(),
    };
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
    fn sample_a(#[case] seed: u128, #[case] location: u128) {
        let input = include_str!("../../sample/fifth.txt");
        let almanac = Almanac::from_str(input).unwrap();
        assert!(almanac.seeds.contains(&seed));
        assert_eq!(location, almanac.location(seed));
    }

    impl FromStr for LookupTable {
        type Err = anyhow::Error;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(parse_lookup_table(s)
                .finish()
                .map_err(|e| anyhow!("{e}"))?
                .1)
        }
    }

    #[rstest]
    #[case(0, 0)]
    #[case(1, 1)]
    #[case(48, 48)]
    #[case(49, 49)]
    #[case(50, 52)]
    #[case(51, 53)]
    #[case(96, 98)]
    #[case(97, 99)]
    #[case(98, 50)]
    #[case(99, 51)]
    #[case(79, 81)]
    #[case(14, 14)]
    #[case(55, 57)]
    #[case(13, 13)]
    fn seed_to_soil_lookup(#[case] seed: u128, #[case] soil: u128) {
        let lut = LookupTable::from_str("50 98 2\n52 50 48").unwrap();
        assert_eq!(soil, lut.lookup(seed));
    }
}
