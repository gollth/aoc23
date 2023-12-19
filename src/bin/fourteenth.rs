use std::{fmt::Debug, str::FromStr};

use aoc23::{
    cycle,
    fourteenth::{animation, Platform, CYCLE, NORTH},
    Part,
};

use anyhow::Result;
use clap::Parser;

/// Day 14: Parabolic Reflector Dish
#[derive(Debug, Parser)]
struct Options {
    /// Path to the file with the input data
    #[clap(short, long, default_value = "sample/fourteenth.txt")]
    input: String,

    /// Which part of the day to solve
    part: Part,

    /// Should the solution be animated?
    #[clap(short, long)]
    animate: bool,

    /// In the animation what is the maximum load you expect for one column of rocks?
    #[clap(short, long, default_value_t = 30.)]
    max_load: f32,
}

fn main() -> Result<()> {
    let args = Options::parse();
    let input = std::fs::read_to_string(args.input)?;
    let mut platform = Platform::from_str(&input)?;

    if args.animate {
        animation::run(platform, args.max_load);
        return Ok(());
    }

    let mut states = Vec::new();

    let solution = match args.part {
        Part::One => {
            platform.tilt(NORTH);
            platform.total_north_load()
        }
        Part::Two => {
            let until = loop {
                for dir in CYCLE.iter() {
                    platform.tilt(*dir);
                }
                states.push(platform.total_north_load());

                if let Some((mu, lambda)) = cycle(states.iter()) {
                    break ((1_000_000_000 - mu) % lambda) + mu;
                }
            };

            // Reset
            platform = Platform::from_str(&input)?;
            for _ in 0..until {
                for dir in CYCLE.iter() {
                    platform.tilt(*dir);
                }
            }
            platform.total_north_load()
        }
    };

    println!("Solution part {:?} {solution}", args.part);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoc23::{
        fourteenth::{EAST, NORTH, SOUTH, WEST},
        Coord,
    };
    use rstest::rstest;

    #[rstest]
    fn sample_a() {
        let input = include_str!("../../sample/fourteenth.txt");
        let mut platform = Platform::from_str(input).expect("parsing");

        platform.tilt(NORTH);
        assert_eq!(136, platform.total_north_load(), "Platform:\n{platform}");
    }

    #[rstest]
    #[case(
        NORTH,
        "OOOO.#.O..
         OO..#....#
         OO..O##..O
         O..#.OO...
         ........#.
         ..#....#.#
         ..O..#.O.O
         ..O.......
         #....###..
         #....#...."
    )]
    #[case(
        SOUTH,
        ".....#....
         ....#....#
         ...O.##...
         ...#......
         O.O....O#O
         O.#..O.#.#
         O....#....
         OO....OO..
         #OO..###..
         #OO.O#...O"
    )]
    #[case(
        EAST,
        "....O#....
         .OOO#....#
         .....##...
         .OO#....OO
         ......OO#.
         .O#...O#.#
         ....O#..OO
         .........O
         #....###..
         #..OO#...."
    )]
    #[case(
        WEST,
        "O....#....
         OOO.#....#
         .....##...
         OO.#OO....
         OO......#.
         O.#O...#.#
         O....#OO..
         O.........
         #....###..
         #OO..#...."
    )]
    fn sample_a_manual(#[case] tilt_dir: Coord, #[case] expected: Platform) {
        let input = include_str!("../../sample/fourteenth.txt");
        let mut platform = Platform::from_str(input).expect("parsing");

        platform.tilt(tilt_dir);
        assert_eq!(
            expected.round_rocks(),
            platform.round_rocks(),
            "Platform:\n{platform}\n\nExpected\n{expected}"
        );
    }

    #[rstest]
    #[case(
        1,
        ".....#....
         ....#...O#
         ...OO##...
         .OO#......
         .....OOO#.
         .O#...O#.#
         ....O#....
         ......OOOO
         #...O###..
         #..OO#...."
    )]
    #[case(
        2,
        ".....#....
         ....#...O#
         .....##...
         ..O#......
         .....OOO#.
         .O#...O#.#
         ....O#...O
         .......OOO
         #..OO###..
         #.OOO#...O"
    )]
    #[case(
        3,
        ".....#....
         ....#...O#
         .....##...
         ..O#......
         .....OOO#.
         .O#...O#.#
         ....O#...O
         .......OOO
         #...O###.O
         #.OOO#...O"
    )]
    fn sample_b_manual(#[case] cycles: usize, #[case] expected: Platform) {
        let input = include_str!("../../sample/fourteenth.txt");
        let mut platform = Platform::from_str(input).expect("parsing");

        for dir in CYCLE.iter().cycle().take(CYCLE.len() * cycles) {
            platform.tilt(*dir);
        }
        assert_eq!(
            expected, platform,
            "Platform:\n{platform}\n\nExpected\n{expected}"
        );
    }
}
