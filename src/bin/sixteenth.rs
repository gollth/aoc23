#![feature(let_chains)]

use std::{fmt::Debug, str::FromStr};

use anyhow::anyhow;
use aoc23::{
    sixteenth::{animation, Contraption, PART_ONE_ENTRY},
    Direction, Part,
};
use clap::Parser;
use rayon::{iter::repeat as par_repeat, prelude::*};

/// Day 16: The Floor Will Be Lava
#[derive(Debug, Parser)]
struct Options {
    /// Path to the file with the input data
    #[clap(short, long, default_value = "sample/sixteenth.txt")]
    input: String,

    /// Which part of the day to solve
    part: Part,

    /// Should the solution be animated?
    #[clap(short, long)]
    animate: bool,

    #[clap(long, short, default_value_t = 50.)]
    frequency: f32,
}

fn main() -> anyhow::Result<()> {
    let args = Options::parse();
    let input = std::fs::read_to_string(args.input)?;

    let mut contraption = Contraption::from_str(&input)?;
    match args.part {
        Part::One => contraption.set_entry(PART_ONE_ENTRY)?,
        Part::Two => {
            let best_entry = par_repeat(Direction::Right)
                .zip(0..contraption.nrows())
                .chain(par_repeat(Direction::Up).zip(0..contraption.ncols()))
                .chain(
                    par_repeat(Direction::Left)
                        .zip(0..contraption.nrows())
                        .rev(),
                )
                .chain(
                    par_repeat(Direction::Down)
                        .zip(0..contraption.ncols())
                        .rev(),
                )
                .map(|entry| {
                    let mut contraption = Contraption::from_str(&input).expect("parsing");
                    contraption.set_entry(entry).unwrap();

                    while !contraption.is_in_equilibrium() {
                        contraption.advance(0.);
                    }
                    (entry, contraption.energized_cells().len())
                })
                .max_by_key(|(_, energized_cells)| *energized_cells)
                .ok_or(anyhow!("No best entry found"))?;
            println!(
                "Found best entry at {:?} leading to {} energized cells",
                best_entry.0, best_entry.1
            );

            contraption.reset();
            contraption.set_entry(best_entry.0)?;
        }
    };

    if args.animate {
        animation::run(contraption, args.frequency);
        return Ok(());
    }

    while !contraption.is_in_equilibrium() {
        contraption.advance(0.);
    }

    let solution = contraption.energized_cells().len();
    println!("Solution: {solution}");

    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(46, PART_ONE_ENTRY, include_str!("../../sample/sixteenth.txt"))]
    #[case(
        9,
        PART_ONE_ENTRY,
        "..|..
         .....
         ..-.."
    )]
    #[case(
        18,
        PART_ONE_ENTRY,
        r#"...\...
           .......
           -......
           .......
           \../..."#
    )]
    #[case(
        16,
        PART_ONE_ENTRY,
        "|....-
         ......
         ......
         -....|"
    )]
    #[case(
        41,
        PART_ONE_ENTRY,
        r#"......|...\..\...
           ..../........|...
           ....\.-.../......
           ......|....../...
           ................."#
    )]
    #[case(51, (Direction::Down,3), include_str!("../../sample/sixteenth.txt"))]
    fn sample(#[case] expectation: usize, #[case] entry: (Direction, i32), #[case] input: &str) {
        let mut max_steps = 100;
        let mut contraption = Contraption::from_str(input).expect("parsing");
        contraption.set_entry(entry).expect("setting entry");
        println!(
            "Contraption Size: {}/{}",
            contraption.ncols(),
            contraption.nrows()
        );
        while !contraption.is_in_equilibrium() {
            contraption.advance(0.);
            println!("{contraption:?}");
            println!(
                "Beams: {:?}",
                contraption
                    .active_beams()
                    .map(|beam| (beam.tip().direction, beam.tip().coord.x, beam.tip().coord.y))
                    .collect::<Vec<_>>()
            );
            if max_steps == 0 {
                panic!("Reached max steps, propably infinite loop");
            }
            max_steps -= 1;
        }
        assert_eq!(expectation, contraption.energized_cells().len())
    }

    #[rstest]
    fn sample_b() {
        let input = include_str!("../../sample/sixteenth.txt");
        let contraption = Contraption::from_str(input).expect("parsing");
        let best_entry = par_repeat(Direction::Right)
            .zip(0..contraption.nrows())
            .chain(par_repeat(Direction::Up).zip(0..contraption.ncols()))
            .chain(
                par_repeat(Direction::Left)
                    .zip(0..contraption.nrows())
                    .rev(),
            )
            .chain(
                par_repeat(Direction::Down)
                    .zip(0..contraption.ncols())
                    .rev(),
            )
            .map(|entry| {
                let mut contraption = Contraption::from_str(input).expect("parsing");
                contraption.set_entry(entry).unwrap();

                while !contraption.is_in_equilibrium() {
                    contraption.advance(0.);
                }
                (entry, contraption.energized_cells().len())
            })
            .max_by_key(|(_, energized_cells)| *energized_cells);

        assert_eq!(Some(((Direction::Down, 3), 51)), best_entry);
    }
}
