use aoc23::{
    ten::{animation, Maze},
    Part,
};

use clap::Parser;
use std::{fmt::Debug, str::FromStr};

/// Day 10: Pipe Maze
#[derive(Debug, Parser)]
struct Options {
    /// Path to the file with the input data
    #[clap(short, long, default_value = "sample/tenth-b.txt")]
    input: String,

    /// Which part of the day to solve
    part: Part,

    /// Print the maze to stdout
    #[clap(short, long)]
    verbose: bool,

    /// Invert the "inside" of the search
    #[clap(long)]
    invert: bool,

    /// Should the solution be animated?
    #[clap(short, long)]
    animate: bool,

    /// How often to execute each step (Hz)
    #[clap(short, long, default_value_t = 5.)]
    frequency: f32,
}

fn main() -> anyhow::Result<()> {
    let args = Options::parse();
    let input = std::fs::read_to_string(&args.input)?;
    let mut maze = Maze::from_str(&input)?;
    let solution = match args.part {
        Part::One => {
            maze.calculate_path();
            maze.path().len() / 2
        }
        Part::Two => {
            maze.calculate_path();
            maze.calculate_inside(args.invert);
            maze.inside().len()
        }
    };

    if args.verbose {
        println!("{maze:?}");
    }

    println!("Solution part {:?}: {solution}", args.part);

    if args.animate {
        animation::run(maze, args.frequency);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case(include_str!("../../sample/tenth-a.txt"), 4)]
    #[case(include_str!("../../sample/tenth-b.txt"), 8)]
    fn sample_a(#[case] s: &str, #[case] expected_distance: usize) {
        let mut maze = Maze::from_str(s).expect("parsing");
        maze.calculate_path();
        println!("{maze:?}");
        assert_eq!(expected_distance, maze.path().len() / 2);
    }

    #[rstest]
    #[case(include_str!("../../sample/tenth-a.txt"), false, 1)]
    #[case(include_str!("../../sample/tenth-b.txt"), false, 1)]
    #[case(include_str!("../../sample/tenth-c.txt"), false, 4)]
    #[case(include_str!("../../sample/tenth-d.txt"), false, 4)]
    #[case(include_str!("../../sample/tenth-e.txt"), true, 8)]
    #[case(include_str!("../../sample/tenth-f.txt"), false, 35)]
    fn sample_b(#[case] s: &str, #[case] ccw: bool, #[case] expected_inside_area: usize) {
        let mut maze = Maze::from_str(s).expect("parsing");
        maze.calculate_inside(ccw);
        println!("{maze:?}");
        assert_eq!(expected_inside_area, maze.inside().len());
    }
}
