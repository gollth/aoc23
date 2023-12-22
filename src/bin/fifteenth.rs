use std::hash::Hasher;

use anyhow::Result;
use aoc23::Part;
use clap::Parser;
use derive_more::{Add, AsRef, From, Into, Sum};

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
                    .map(|hash| *hash.as_ref() as usize)
                    .sum::<usize>()
            })
            .sum::<usize>(),
        Part::Two => unimplemented!(),
    };
    println!("Solution part {:?}: {solution}", args.part);

    Ok(())
}

#[derive(Debug, Default, PartialEq, Eq, From, Into, Add, Sum, AsRef)]
#[allow(clippy::upper_case_acronyms)]
struct HASH(u8);

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
}
