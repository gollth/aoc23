use std::{collections::HashMap, hash::Hasher, iter::repeat};

use anyhow::Result;
use aoc23::{anyhowing, Part};
use clap::Parser;
use derive_more::{Add, AsRef, From, Into, Sum};
use itertools::izip;
use nom::{
    character::complete::{alpha1, char, digit1},
    multi::separated_list1,
    sequence::tuple,
    Finish, IResult, Parser as NomParser,
};
use nom_supreme::ParserExt;

/// Day 15: Lens Library
#[derive(Debug, Parser)]
struct Options {
    /// Path to the file with the input data
    #[clap(short, long, default_value = "sample/fifteenth.txt")]
    input: String,

    /// Which part of the day to solve
    part: Part,
}

fn main() -> Result<()> {
    let args = Options::parse();
    let input = std::fs::read_to_string(args.input)?;

    let solution = match args.part {
        Part::One => input
            .lines()
            .map(|line| {
                line.split(',')
                    .map(|chunk| chunk.bytes().collect::<HASH>())
                    .map(|hash| *hash.as_ref() as u64)
                    .sum::<u64>()
            })
            .sum::<u64>(),
        Part::Two => {
            let facility = Facility::from_str(&input)?;
            facility.focal_power()
        }
    };
    println!("Solution part {:?}: {solution}", args.part);

    Ok(())
}

type Label<'a> = &'a str;
type FocalLength = u64;
type Box<'a> = Vec<(Label<'a>, FocalLength)>;
type Instruction<'a> = (Label<'a>, Operation);

struct Facility<'a>(HashMap<u8, Box<'a>>);

impl<'a> FromIterator<Instruction<'a>> for Facility<'a> {
    fn from_iter<T: IntoIterator<Item = Instruction<'a>>>(iter: T) -> Self {
        Self(
            iter.into_iter()
                .fold(HashMap::new(), |mut map, (label, operation)| {
                    // println!("Label {label} ({}): {operation:?}", hash(label));
                    match operation {
                        Operation::Remove => {
                            if let Some(box_) = map.get_mut(&hash(label)) {
                                box_.retain(|lens| lens.0 != label);
                            }
                        }
                        Operation::Insert(fl) => {
                            let box_ = map.entry(hash(label)).or_default();
                            match box_.iter_mut().find(|(l, _)| label == *l) {
                                Some(lens) => lens.1 = fl,
                                None => box_.push((label, fl)),
                            }
                        }
                    };
                    map
                }),
        )
    }
}

impl<'a> Facility<'a> {
    fn from_str(s: &'a str) -> Result<Self> {
        Ok(instructions(s)
            .finish()
            .map_err(anyhowing)?
            .1
            .into_iter()
            .collect())
    }

    fn focal_power(&self) -> u64 {
        self.0
            .iter()
            .flat_map(|(box_, lenses)| {
                izip!(repeat(1 + *box_ as u64), 1.., lenses)
                    .map(|(box_nr, slot, (_, focal_length))| box_nr * slot * focal_length)
                // .inspect(|x| println!(">> {x:?}"))
            })
            .sum()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Operation {
    Remove,
    Insert(FocalLength),
}

fn operation(s: &str) -> IResult<&str, Operation> {
    char('-')
        .value(Operation::Remove)
        .or(char('=')
            .precedes(digit1.map_res(str::parse))
            .map(Operation::Insert))
        .parse(s)
}

fn label(s: &str) -> IResult<&str, Label<'_>> {
    alpha1(s)
}
fn instruction(s: &str) -> IResult<&str, (Label<'_>, Operation)> {
    tuple((label, operation)).parse(s)
}

fn instructions(s: &str) -> IResult<&str, Vec<(Label<'_>, Operation)>> {
    separated_list1(char(','), instruction).parse(s)
}

#[derive(Debug, Default, PartialEq, Eq, From, Into, Add, Sum, AsRef)]
#[allow(clippy::upper_case_acronyms)]
struct HASH(u8);

fn hash(s: &str) -> u8 {
    let mut h = HASH::default();
    h.write(s.as_bytes());
    h.finish() as u8
}

impl Hasher for HASH {
    fn finish(&self) -> u64 {
        self.0.into()
    }

    fn write_u8(&mut self, x: u8) {
        self.0 = self.0.wrapping_add(x).wrapping_mul(17);
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.write_u8(*byte);
        }
    }
}

impl<T> FromIterator<T> for HASH
where
    T: Into<u8>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        iter.into_iter().fold(HASH::default(), |mut hash, x| {
            hash.write_u8(x.into());
            hash
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    fn sample_a_hash() {
        assert_eq!(
            52,
            "HASH"
                .lines()
                .map(|line| line.bytes().collect::<HASH>().finish())
                .sum::<u64>()
        );
    }

    #[rstest]
    fn sample_a() {
        let input = include_str!("../../sample/fifteenth.txt");
        assert_eq!(
            1320,
            input
                .lines()
                .map(|line| line
                    .split(',')
                    .map(|chunk| chunk.bytes().collect::<HASH>().finish())
                    .sum::<u64>())
                .sum::<u64>()
        );
    }

    #[rstest]
    #[case("rn=1", Ok(("",("rn", Operation::Insert(1)))))]
    #[case("cm-", Ok(("",("cm", Operation::Remove))))]
    #[case("qp=3", Ok(("",("qp", Operation::Insert(3)))))]
    #[case("foobar=3,blub", Ok((",blub",("foobar", Operation::Insert(3)))))]
    fn sample_b_parsing(#[case] input: &str, #[case] expected: IResult<&str, (&str, Operation)>) {
        assert_eq!(expected, instruction(input));
    }

    #[rstest]
    fn sample_b() {
        let input = include_str!("../../sample/fifteenth.txt");
        let facility = Facility::from_str(input).expect("parsing");
        assert_eq!(145, facility.focal_power());
    }
}
