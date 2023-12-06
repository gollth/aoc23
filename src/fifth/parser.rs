use super::{Almanac, Mapping, Resource};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, i128, line_ending, space1},
    combinator::map,
    multi::{many_till, separated_list1},
    sequence::{preceded, separated_pair, terminated, tuple},
    IResult, Parser as NomParser,
};
use std::ops::Range;

pub(crate) fn parse_seeds_individual(s: &str) -> IResult<&str, Vec<Range<i128>>> {
    separated_list1(space1, map(i128, |x| x..(x + 1)))(s)
}

pub(crate) fn parse_seeds_ranges(s: &str) -> IResult<&str, Vec<Range<i128>>> {
    separated_list1(
        space1,
        map(separated_pair(i128, space1, i128), |(a, b)| a..(a + b)),
    )(s)
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

pub(crate) fn parse_almanac(s: &str) -> IResult<&str, Almanac> {
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
