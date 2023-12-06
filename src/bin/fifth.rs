use aoc23::{fifth::Almanac, Part};

use anyhow::Result;
use clap::Parser;

/// Day 5: If You Give A Seed A Fertilizer
#[derive(Debug, Parser)]
struct Options {
    /// Path to the file with the input data
    #[clap(short, long, default_value = "sample/fifth.txt")]
    input: String,

    /// Which part of the day to solve
    part: Part,
}

fn main() -> Result<()> {
    let args = Options::parse();
    let input = std::fs::read_to_string(args.input)?;
    let (almanac, seeds) = Almanac::parse(args.part, &input)?;
    let solution = almanac.best_location(&seeds);
    println!("Solution part {:?}: {solution}", args.part);
    Ok(())
}
