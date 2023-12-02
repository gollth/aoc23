use std::{collections::HashMap, str::FromStr};

use aoc23::second::{Color, Game};
use clap::Parser;
use lazy_static::lazy_static;

#[derive(Debug, Parser)]
struct Options {
    /// Path to the file with the input data
    #[clap(short, long, default_value = "sample/second.txt")]
    input: String,
}

lazy_static! {
    static ref BAG: HashMap<Color, u32> =
        vec![(Color::Red, 12), (Color::Green, 13), (Color::Blue, 14)]
            .into_iter()
            .collect();
}

fn possible_game_ids(input: &str) -> impl Iterator<Item = u32> + '_ {
    input
        .lines()
        .filter_map(|line| Game::from_str(line).ok())
        .filter(|game| game.possible(&BAG))
        .map(|game| game.id())
}

fn main() -> anyhow::Result<()> {
    let args = Options::parse();
    let input = std::fs::read_to_string(args.input)?;

    let answer = possible_game_ids(&input).sum::<u32>();
    println!("Solution A: {answer}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_a() {
        let sample = include_str!("../../sample/second.txt");
        assert_eq!(vec![1, 2, 5], possible_game_ids(sample).collect::<Vec<_>>())
    }
}
