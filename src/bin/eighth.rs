use aoc23::{anyhowing, Part};

use anyhow::Result;
use clap::Parser;
use nom::{
    character::complete::{alpha1, char, multispace1, newline, space0},
    multi::{many_till, separated_list1},
    sequence::{separated_pair, tuple},
    Finish, IResult, Parser as NomParser,
};
use nom_supreme::ParserExt;
use std::{collections::HashMap, iter::Cycle, vec::IntoIter};

/// Day 8: Haunted Wasteland
#[derive(Debug, Parser)]
struct Options {
    /// Path to the file with the input data
    #[clap(short, long, default_value = "sample/eighth.txt")]
    input: String,

    /// Which part of the day to solve
    part: Part,

    /// Print the path to stdout
    #[clap(long, short)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Options::parse();

    let input = std::fs::read_to_string(&args.input)?;
    let map = Map::new(&input)?;
    let solution = match args.part {
        Part::One => {
            map.into_iter()
                .enumerate()
                .inspect(|(i, node)| {
                    if args.verbose {
                        println!("#[{i:0>5}] {node}")
                    }
                })
                .count()
                - 1
        }
        Part::Two => unimplemented!(),
    };
    println!("Solution part {part:?}: {solution}", part = args.part);
    Ok(())
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Direction {
    L,
    R,
}

type Node<'a> = &'a str;
type Instructions = Cycle<IntoIter<Direction>>;
type Network<'a> = HashMap<Node<'a>, (Node<'a>, Node<'a>)>;

#[derive(Debug)]
struct Map<'a> {
    network: Network<'a>,
    instructions: Instructions,
}
impl<'a> Map<'a> {
    fn new(s: &'a str) -> Result<Self> {
        Ok(parse_map(s).finish().map_err(anyhowing)?.1)
    }
}

struct MapIter<'a> {
    node: Option<Node<'a>>,
    network: Network<'a>,
    instructions: Instructions,
}

impl<'a> IntoIterator for Map<'a> {
    type Item = Node<'a>;
    type IntoIter = MapIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        MapIter {
            node: None,
            instructions: self.instructions,
            network: self.network,
        }
    }
}

impl<'a> Iterator for MapIter<'a> {
    type Item = Node<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.node = match self.node {
            None => Some("AAA"),
            Some("ZZZ") => None,
            Some(node) => {
                let dir = self.instructions.next()?;
                let (left, right) = self.network.get(node)?;
                Some(match dir {
                    Direction::L => left,
                    Direction::R => right,
                })
            }
        };
        self.node
    }
}

fn instructions(s: &str) -> IResult<&str, Cycle<IntoIter<Direction>>> {
    let left = char('L').value(Direction::L);
    let right = char('R').value(Direction::R);
    many_till(left.or(right), multispace1)
        .map(|(dirs, _)| dirs.into_iter().cycle())
        .parse(s)
}

fn node(s: &str) -> IResult<&str, Node<'_>> {
    alpha1(s)
}
fn network(s: &str) -> IResult<&str, HashMap<Node, (Node, Node)>> {
    separated_list1(
        newline,
        separated_pair(
            node,
            space0.and(char('=')).and(space0),
            char('(')
                .precedes(separated_pair(node, char(',').and(space0), node))
                .terminated(char(')')),
        ),
    )
    .map(HashMap::from_iter)
    .parse(s)
}
fn parse_map(s: &str) -> IResult<&str, Map<'_>> {
    tuple((instructions, network))
        .map(|(instructions, network)| Map {
            instructions,
            network,
        })
        .parse(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;
    use rstest::rstest;

    const NETWORK_SIMPLE: &str = indoc! {"LR
        AAA = (BBB, CCC)
    "};
    const NETWORK_THREE_NODES: &str = indoc! {"LR
        AAA = (BBB, CCC)
        BBB = (KJL, ABC)
        CCC = (ZZZ, FOO)
    "};
    const NETWORK_SEVEN_NODES: &str = indoc! {"RL
       AAA = (BBB, CCC)
       BBB = (DDD, EEE)
       CCC = (ZZZ, GGG)
       DDD = (DDD, DDD)
       EEE = (EEE, EEE)
       GGG = (GGG, GGG)
       ZZZ = (ZZZ, ZZZ)
     "};
    const NETWORK_SAMPLE: &str = include_str!("../../sample/eighth.txt");

    #[rstest]
    #[case(NETWORK_SIMPLE, vec![("AAA", ("BBB", "CCC"))])]
    #[case(NETWORK_THREE_NODES, vec![
            ("AAA", ("BBB", "CCC")),
            ("BBB", ("KJL", "ABC")),
            ("CCC", ("ZZZ", "FOO")),
        ])
    ]
    fn map_from_str(#[case] map: &str, #[case] expected_network: Vec<(&str, (&str, &str))>) {
        let map = Map::new(map).expect("parsing");
        for (node, (l, r)) in expected_network {
            assert!(
                map.network.get(node).is_some(),
                "Expected node {node} to be present in network"
            );
            assert_eq!(Some(&(l, r)), map.network.get(node))
        }
    }

    #[rstest]
    #[case(NETWORK_SEVEN_NODES, vec!["AAA", "CCC", "ZZZ"])]
    #[case(NETWORK_SAMPLE, vec!["AAA", "BBB", "AAA", "BBB", "AAA", "BBB", "ZZZ"])]
    fn sample_a(#[case] map: &str, #[case] expected_path: Vec<&str>) {
        let map = Map::new(map).expect("parsing");
        assert_eq!(expected_path, map.into_iter().collect::<Vec<_>>());
    }
}
