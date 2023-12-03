use std::{collections::HashMap, str::FromStr};

use aoc23::{
    second::{Color, Game},
    Part,
};
use clap::Parser;
use lazy_static::lazy_static;

/// Day 2: Cube Conundrum
#[derive(Debug, Parser)]
struct Options {
    /// Path to the file with the input data
    #[clap(short, long, default_value = "sample/second.txt")]
    input: String,

    /// Which part of the day to solve
    part: Part,
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
fn powers(input: &str) -> impl Iterator<Item = u32> + '_ {
    input
        .lines()
        .filter_map(|line| Game::from_str(line).ok())
        .map(|game| game.fewest())
        .map(|f| {
            f.get(&Color::Red).unwrap_or(&0)
                * f.get(&Color::Green).unwrap_or(&0)
                * f.get(&Color::Blue).unwrap_or(&0)
        })
}

fn main() -> anyhow::Result<()> {
    let args = Options::parse();
    let input = std::fs::read_to_string(args.input)?;

    let answer = match args.part {
        Part::One => possible_game_ids(&input).sum::<u32>(),
        Part::Two => powers(&input).sum(),
    };
    println!("Solution Part {:?}: {answer}", args.part);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_part_one() {
        let sample = include_str!("../../sample/second.txt");
        assert_eq!(vec![1, 2, 5], possible_game_ids(sample).collect::<Vec<_>>())
    }

    #[test]
    fn sample_part_two() {
        let sample = include_str!("../../sample/second.txt");
        assert_eq!(
            vec![48, 12, 1560, 630, 36],
            powers(sample).collect::<Vec<_>>()
        );
    }
}
