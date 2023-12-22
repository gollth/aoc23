use std::{array, hash::Hasher, iter::repeat};

use crate::anyhowing;
use anyhow::Result;
use derive_more::{Add, AsRef, From, Into, Sum};
use itertools::izip;
use nom::Finish;

use self::parser::instructions;

mod animation;
mod parser;

type Label<'a> = &'a str;
type FocalLength = u64;
type Box<'a> = Vec<(Label<'a>, FocalLength)>;
type Instruction<'a> = (Label<'a>, Operation);

const N: usize = 256;
pub struct HashMap<'a>([Box<'a>; N]);

impl<'a> FromIterator<Instruction<'a>> for HashMap<'a> {
    fn from_iter<T: IntoIterator<Item = Instruction<'a>>>(iter: T) -> Self {
        Self(iter.into_iter().fold(
            array::from_fn(|_| Vec::default()),
            |mut map, (label, operation)| {
                match operation {
                    Operation::Remove => {
                        map[hash(label)].retain(|lens| lens.0 != label);
                    }
                    Operation::Insert(fl) => {
                        let item = &mut map[hash(label)];
                        match item.iter_mut().find(|(l, _)| label == *l) {
                            Some(lens) => lens.1 = fl,
                            None => item.push((label, fl)),
                        }
                    }
                };
                map
            },
        ))
    }
}

impl<'a> HashMap<'a> {
    #[allow(clippy::should_implement_trait)] // whole point is to be similar. Unable to implement
                                             // FromStr directly because of lifetime
    pub fn from_str(s: &'a str) -> Result<Self> {
        Ok(instructions(s)
            .finish()
            .map_err(anyhowing)?
            .1
            .into_iter()
            .collect())
    }

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
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Operation {
    Remove,
    Insert(FocalLength),
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
    #[case("rn=1", Ok(("",("rn", Operation::Insert(1)))))]
    #[case("cm-", Ok(("",("cm", Operation::Remove))))]
    #[case("qp=3", Ok(("",("qp", Operation::Insert(3)))))]
    #[case("foobar=3,blub", Ok((",blub",("foobar", Operation::Insert(3)))))]
    fn sample_b_parsing(#[case] input: &str, #[case] expected: IResult<&str, (&str, Operation)>) {
        assert_eq!(expected, instruction(input));
    }
}
