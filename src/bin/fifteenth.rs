use std::{hash::Hasher, str::FromStr};

use anyhow::{anyhow, Result};
use aoc23::{
    fifteenth::{animation, HashMap, HASH},
    Part,
};
use clap::Parser;

/// Day 15: Lens Library
#[derive(Debug, Parser)]
struct Options {
    /// Path to the file with the input data
    #[clap(short, long, default_value = "sample/fifteenth.txt")]
    input: String,

    /// Which part of the day to solve
    part: Part,

    /// Should the solution be animated?
    #[clap(short, long)]
    animate: bool,

    /// How fast shall the animation run initially
    #[clap(short, long, default_value_t = 1.5)]
    frequency: f32,
}

fn main() -> Result<()> {
    let args = Options::parse();
    let input = std::fs::read_to_string(args.input)?;
    let solution = match args.part {
        Part::One if args.animate => return Err(anyhow!("Part one cannot be animated")),
        Part::One => input
            .lines()
            .map(|line| {
                line.split(',')
                    .map(|chunk| chunk.bytes().collect::<HASH>().finish())
                    .sum::<u64>()
            })
            .sum::<u64>(),
        Part::Two => {
            if args.animate {
                animation::run(args.frequency, HashMap::default(), &input);
                0
            } else {
                let facility = HashMap::from_str(&input)?;
                facility.focal_power()
            }
        }
    };
    println!("Solution part {:?}: {solution}", args.part);
    Ok(())
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
    fn sample_b() {
        let input = include_str!("../../sample/fifteenth.txt");
        let facility = HashMap::from_str(input).expect("parsing");
        assert_eq!(145, facility.focal_power());
    }
}
