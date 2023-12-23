use std::{array, fmt::Display, hash::Hasher, iter::repeat, str::FromStr};

use crate::anyhowing;
use anyhow::Result;
use bevy::ecs::system::Resource;
use derive_more::{Add, AsRef, From, Into, Sum};
use itertools::izip;
use nom::Finish;

use self::parser::instructions;

pub mod animation;
mod parser;

type Label = String;
type FocalLength = u64;
type Box = Vec<(Label, FocalLength)>;
type Instruction = (Label, Operation);

pub(crate) const N: usize = 256;

#[derive(Debug, Resource)]
pub struct HashMap([Box; N]);

impl FromIterator<Instruction> for HashMap {
    fn from_iter<T: IntoIterator<Item = Instruction>>(iter: T) -> Self {
        let mut me = Self::default();
        for instruction in iter {
            me.process(instruction);
        }
        me
    }
}
impl Default for HashMap {
    fn default() -> Self {
        Self(array::from_fn(|_| Vec::default()))
    }
}

impl FromStr for HashMap {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(instructions(s)
            .finish()
            .map_err(anyhowing)?
            .1
            .into_iter()
            .collect())
    }
}

impl HashMap {
    pub fn focal_power(&self) -> u64 {
        self.0
            .iter()
            .enumerate()
            .flat_map(|(box_, lenses)| {
                izip!(repeat(1 + box_ as u64), 1.., lenses)
                    .map(|(box_nr, slot, (_, focal_length))| box_nr * slot * focal_length)
            })
            .sum()
    }

    pub fn get(&self, key: &str) -> impl Iterator<Item = &(Label, FocalLength)> {
        self.index(hash(key) as u8)
    }
    pub fn index(&self, i: u8) -> impl Iterator<Item = &(Label, FocalLength)> {
        self.0[i as usize].iter()
    }

    pub(crate) fn process(&mut self, (label, operation): Instruction) {
        match operation {
            Operation::Remove => {
                self.0[hash(&label)].retain(|lens| lens.0 != label);
            }
            Operation::Insert(fl) => {
                let item = &mut self.0[hash(&label)];
                match item.iter_mut().find(|(l, _)| label == *l) {
                    Some(lens) => lens.1 = fl,
                    None => item.push((label, fl)),
                }
            }
        };
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum Operation {
    Remove,
    Insert(FocalLength),
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Remove => write!(f, "-"),
            Self::Insert(l) => write!(f, "={l}"),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, From, Into, Add, Sum, AsRef)]
#[allow(clippy::upper_case_acronyms)]
pub struct HASH(u8);

fn hash(s: &str) -> usize {
    let mut h = HASH::default();
    h.write(s.as_bytes());
    h.finish() as usize
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
    use crate::fifteenth::parser::instruction;
    use nom::IResult;
    use rstest::rstest;

    #[rstest]
    #[case("rn=1", Ok(("",(String::from("rn"), Operation::Insert(1)))))]
    #[case("cm-", Ok(("",(String::from("cm"), Operation::Remove))))]
    #[case("qp=3", Ok(("",(String::from("qp"), Operation::Insert(3)))))]
    #[case("foobar=3,blub", Ok((",blub",(String::from("foobar"), Operation::Insert(3)))))]
    fn sample_b_parsing(#[case] input: &str, #[case] expected: IResult<&str, (String, Operation)>) {
        assert_eq!(expected, instruction(input));
    }
}
