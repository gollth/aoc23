use aoc23::{anyhowing, Part};

use anyhow::Result;
use clap::Parser;
use itertools::Itertools;
use nom::{
    character::complete::{alphanumeric1, char, multispace1, newline, space0},
    multi::{many_till, separated_list1},
    sequence::{separated_pair, tuple},
    Finish, IResult, Parser as NomParser,
};
use nom_supreme::ParserExt;
use std::{
    collections::HashMap,
    iter::{repeat, Cycle},
    vec::IntoIter,
};

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
    let map = Map::new(&input, args.part)?;
    let solution = match args.part {
        Part::One => {
            map.into_iter()
                .enumerate()
                .inspect(|(i, node)| {
                    if args.verbose {
                        println!("#[{i:0>5}] {node:?}")
                    }
                })
                .count()
                // subtract start node from calculation
                - 1
        }
        Part::Two => {
            // Dont understand why this works, but seems to be the solution on reddit =(
            let mut memo = HashMap::new();
            let mut found_cycle = repeat(false).take(map.starts.len()).collect::<Vec<_>>();
            for step in map {
                for (i, node) in step.iter().copied().enumerate() {
                    if node.ends_with('Z') {
                        match memo.get(&i) {
                            None => {
                                memo.insert(i, 0);
                            }
                            Some(_) => found_cycle[i] = true,
                        }
                    }
                    if let Some(count) = memo.get_mut(&i) {
                        if !found_cycle[i] {
                            *count += 1;
                        }
                    }
                }
                if found_cycle.iter().all(|x| *x) {
                    break;
                }
            }
            memo.values().copied().reduce(num::integer::lcm).unwrap()
        }
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
    starts: Vec<Node<'a>>,
    network: Network<'a>,
    instructions: Instructions,
}
impl<'a> Map<'a> {
    fn new(s: &'a str, part: Part) -> Result<Self> {
        let (instructions, network) = parse_map(s).finish().map_err(anyhowing)?.1;
        let starts = network
            .keys()
            .copied()
            .filter(|&node| match part {
                Part::One => node == "AAA",
                Part::Two => node.ends_with('A'),
            })
            .sorted()
            .collect();
        Ok(Map {
            instructions,
            network,
            starts,
        })
    }
}

#[derive(Debug)]
struct MapIter<'a> {
    yielded_start: bool,
    state: Vec<Node<'a>>,
    network: Network<'a>,
    instructions: Instructions,
}

impl<'a> IntoIterator for Map<'a> {
    type Item = Vec<Node<'a>>;
    type IntoIter = MapIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        MapIter {
            yielded_start: false,
            state: self.starts,
            instructions: self.instructions,
            network: self.network,
        }
    }
}

impl<'a> Iterator for MapIter<'a> {
    type Item = Vec<Node<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state.iter().all(|node| node.ends_with('Z')) {
            // All ghosts found an end node
            return None;
        }
        if !self.yielded_start {
            self.yielded_start = true;
            return Some(self.state.clone());
        }

        let dir = self.instructions.next()?;
        for node in self.state.iter_mut() {
            // simulation
            let (left, right) = self.network.get(node)?;
            *node = match dir {
                Direction::L => *left,
                Direction::R => *right,
            };
        }

        Some(self.state.clone())
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
    alphanumeric1(s)
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
fn parse_map(s: &str) -> IResult<&str, (Instructions, Network<'_>)> {
    tuple((instructions, network)).parse(s)
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
        let map = Map::new(map, Part::One).expect("parsing");
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
        let map = Map::new(map, Part::One).expect("parsing");
        assert_eq!(expected_path, map.into_iter().flatten().collect::<Vec<_>>());
    }

    const NETWORK_SEVEN_NODES2: &str = indoc! {"LR
        11A = (11B, XXX)
        11B = (XXX, 11Z)
        11Z = (11B, XXX)
        22A = (22B, XXX)
        22B = (22C, 22C)
        22C = (22Z, 22Z)
        22Z = (22B, 22B)
        XXX = (XXX, XXX)
     "};

    #[rstest]
    #[case(NETWORK_SEVEN_NODES2, vec![
        vec!["11A", "11B", "11Z", "11B", "11Z", "11B", "11Z"],
        vec!["22A", "22B", "22C", "22Z", "22B", "22C", "22Z"],
    ])]
    fn sample_b(#[case] map: &str, #[case] expected_paths: Vec<Vec<&str>>) {
        let map = Map::new(map, Part::Two).expect("parsing");
        assert_eq!(
            transpose(expected_paths),
            map.into_iter().collect::<Vec<_>>()
        );
    }

    fn transpose<T>(v: Vec<Vec<T>>) -> Vec<Vec<T>>
    where
        T: Clone,
    {
        assert!(!v.is_empty());
        (0..v[0].len())
            .map(|i| v.iter().map(|inner| inner[i].clone()).collect::<Vec<T>>())
            .collect()
    }
}
