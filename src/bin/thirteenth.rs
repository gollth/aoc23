use std::{fmt::Debug, str::FromStr};

use aoc23::{
    thirteenth::{animation, Grid, Reflection},
    Part,
};

use anyhow::Result;
use clap::Parser;

/// Day 13: Point of Incidence
#[derive(Debug, Parser)]
struct Options {
    /// Path to the file with the input data
    #[clap(short, long, default_value = "sample/thirteenth.txt")]
    input: String,

    /// Which part of the day to solve
    part: Part,

    /// Should the solution be animated?
    #[clap(short, long)]
    animate: bool,

    /// How often to execute each step (Hz)
    #[clap(short, long, default_value_t = 2.)]
    frequency: f32,
}

fn main() -> anyhow::Result<()> {
    let args = Options::parse();
    let input = std::fs::read_to_string(args.input)?;
    let mut grids = input
        .split("\n\n")
        .map(Grid::from_str)
        .collect::<Result<Vec<_>>>()?;

    let mut lefts = 0;
    let mut aboves = 0;

    if args.part == Part::Two {
        for grid in grids.iter_mut() {
            let (_index, fold, dir) = [Reflection::Horizontal, Reflection::Vertical]
                .into_iter()
                .flat_map(|r| grid.find_smudge(r))
                .next()
                .expect("a smudge");
            match dir {
                Reflection::Horizontal => aboves += fold,
                Reflection::Vertical => lefts += fold,
            }
        }
    } else {
        for (dir, x) in grids.iter().flat_map(|grid| {
            grid.fold_line(Reflection::Horizontal)
                .or(grid.fold_line(Reflection::Vertical))
        }) {
            match dir {
                Reflection::Vertical => lefts += x,
                Reflection::Horizontal => aboves += x,
            }
        }
    }
    let solution = lefts + 100 * aboves;
    println!("Solution part {:?}: {solution}", args.part);

    if args.animate {
        animation::run(grids, args.frequency);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case(
        Reflection::Vertical,
        5,
        "
           #.##..##.
           ..#.##.#.
           ##......#
           ##......#
           ..#.##.#.
           ..##..##.
           #.#.##.#."
    )]
    #[case(
        Reflection::Horizontal,
        4,
        "
           #...##..#
           #....#..#
           ..##..###
           #####.##.
           #####.##.
           ..##..###
           #....#..#"
    )]
    #[case(
        Reflection::Vertical,
        7,
        "
           #.#######
           ....#####
           ..##.....
           .#.######
           .#..#....
           #........
           ..#...##.
           #.###....
           ###.##..#
           .########
           #.##.#..#
           .#.###..#
           ..###....
           ..###....
           ...###..#"
    )]
    #[case(
        Reflection::Vertical,
        16,
        "
            #.##.######.##.##
            ###...####...####
            ....##.##.##.....
            #..#.#....#.#..##
            .....##..##......
            #.#.###..###.#.##
            .##.#.####.#.#...
            .#..#......#..#..
            .####.####.####..
            ###...####...####
            #.##........##.##
            .#....#..#....#..
            ..###.####.###...
"
    )]
    fn sample_a_manual(
        #[case] reflection: Reflection,
        #[case] expected_split: usize,
        #[case] grid: Grid,
    ) {
        assert_eq!(
            Some((reflection, expected_split)),
            grid.fold_line(reflection),
            "\n{grid:?}"
        );
    }

    #[rstest]
    #[case(
        Reflection::Horizontal,
        Some(((0,0),3)),
        "
           #.##..##.
           ..#.##.#.
           ##......#
           ##......#
           ..#.##.#.
           ..##..##.
           #.#.##.#."
    )]
    #[case(
        Reflection::Horizontal,
        Some(((0,4),1)),
        "
           #...##..#
           #....#..#
           ..##..###
           #####.##.
           #####.##.
           ..##..###
           #....#..#"
    )]
    #[case(
        Reflection::Horizontal,
        Some(((0,3),1)),
        "
            .#.##
            .#..#"
    )]
    #[case(
        Reflection::Horizontal,
        Some(((1,0), 2)),
        "
            .###.
            .#..#
            ##..#
            .###."
    )]
    #[case(
        Reflection::Horizontal,
        Some(((2,3),4)),
        "
            .###.
            .#..#
            .##..
            ##..#
            ##..#
            .###."
    )]
    #[case(
        Reflection::Vertical,
        Some(((0,2), 3)),
        "
            .#.##
            .#..#"
    )]
    #[case(
        Reflection::Vertical,
        Some(((3,2), 4)),
        "
            ...##.
            ..#..#
            .##..#
            #..###"
    )]
    #[case(
        Reflection::Vertical,
        None,
        "
            .#....#..#...
            #..##..##....
            ...##...#.##.
            ...##...###.#
            .##..##.#.#.#
            .##..##.#.###
            ...##...###.#"
    )]
    #[case(
        Reflection::Horizontal,
        Some(((4,11), 5)),
        "
            .#....#..#...
            #..##..##....
            ...##...#.##.
            ...##...###.#
            .##..##.#.#.#
            .##..##.#.###
            ...##...###.#"
    )]
    #[case(
        Reflection::Horizontal,
        Some(((6,7), 7)),
        "
            #...##.#.##
            #...##.#.##
            .###..#.#.#
            ##.###..#..
            ..#...##..#
            ....##.####
            ##.###.#.#.
            ##.###...#.
            ....##.####
            ..#...##..#
            ##.###..#..
            .###..#.#.#
            #...##.#.##"
    )]
    fn sample_b_manual(
        #[case] reflection: Reflection,
        #[case] expectation: Option<((usize, usize), usize)>,
        #[case] grid: Grid,
    ) {
        let expectation = expectation.map(|(a, b)| (a, b, reflection));
        assert_eq!(
            expectation,
            grid.find_smudge(reflection),
            "split {reflection:?}: \n{:?}",
            grid
        );
    }

    #[rstest]
    fn sample_b() {
        let input = include_str!("../../sample/thirteenth.txt");

        let mut grids = input
            .split("\n\n")
            .map(Grid::from_str)
            .collect::<Result<Vec<_>>>()
            .unwrap();

        let mut lefts = 0;
        let mut aboves = 0;
        for grid in grids.iter_mut() {
            let (_index, fold, dir) = [Reflection::Horizontal, Reflection::Vertical]
                .into_iter()
                .flat_map(|r| grid.find_smudge(r))
                .next()
                .expect("a smudge");
            match dir {
                Reflection::Vertical => lefts += fold,
                Reflection::Horizontal => aboves += fold,
            };
        }

        assert_eq!(400, lefts + 100 * aboves);
    }
}
