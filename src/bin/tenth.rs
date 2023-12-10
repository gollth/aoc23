#![feature(generators, iter_from_generator)]

use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    iter,
    ops::Add,
    str::FromStr,
};

use anyhow::anyhow;
use aoc23::Part;
use enum_iterator::{next_cycle, previous_cycle, Sequence};
use itertools::Itertools;
use termion::color::{Fg, Red, Reset};

use clap::Parser;

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
}

fn main() -> anyhow::Result<()> {
    let args = Options::parse();
    let input = std::fs::read_to_string(&args.input)?;
    let mut maze = Maze::from_str(&input)?;
    let solution = match args.part {
        Part::One => {
            maze.path = maze
                .follow(&maze.start, Direction::Right)
                .take_while_inclusive(|c| *c != maze.start)
                .collect::<Vec<_>>();
            maze.path.len() / 2
        }
        Part::Two => unimplemented!(),
    };

    if args.verbose {
        println!("{maze:?}");
    }
    println!("Solution part {:?}: {solution}", args.part);
    Ok(())
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Hash)]
struct Coord {
    x: i32,
    y: i32,
}
impl Coord {
    fn zero() -> Self {
        Self::default()
    }
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    fn max(&self, other: &Coord) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }
}
impl Add<Direction> for &Coord {
    type Output = Coord;
    fn add(self, d: Direction) -> Self::Output {
        let (x, y) = match d {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        };
        Coord::new(self.x + x, self.y + y)
    }
}

impl Pipe {
    fn follow(&self, d: Direction) -> Option<Direction> {
        match (d, *self) {
            (_, Pipe::NS | Pipe::EW | Pipe::Start) => Some(d),

            (Direction::Down, Pipe::NW) => Some(d.cw()),
            (Direction::Right, Pipe::NW) => Some(d.ccw()),

            (Direction::Down, Pipe::NE) => Some(d.ccw()),
            (Direction::Left, Pipe::NE) => Some(d.cw()),

            (Direction::Right, Pipe::SW) => Some(d.cw()),
            (Direction::Up, Pipe::SW) => Some(d.ccw()),

            (Direction::Left, Pipe::SE) => Some(d.ccw()),
            (Direction::Up, Pipe::SE) => Some(d.cw()),

            _ => None,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Sequence)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    fn cw(&self) -> Self {
        next_cycle(self).unwrap()
    }
    fn ccw(&self) -> Self {
        previous_cycle(self).unwrap()
    }
}

struct Maze {
    pipes: HashMap<Coord, Pipe>,
    start: Coord,
    size: Coord,
    path: Vec<Coord>,
}

impl Maze {
    fn advance(&self, coord: &Coord, direction: Direction) -> Option<(Coord, Direction)> {
        let pipe = self.pipes.get(coord)?;
        let newdir = pipe.follow(direction)?;
        let next = coord + newdir;
        Some((next, newdir))
    }

    fn follow(&self, coord: &Coord, mut dir: Direction) -> impl Iterator<Item = Coord> + '_ {
        let mut coord = coord.clone();
        iter::from_generator(move || {
            while let Some((c, d)) = self.advance(&coord, dir) {
                yield c.clone();
                coord = c;
                dir = d;
            }
            yield coord;
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Pipe {
    NS,
    EW,
    NW,
    NE,
    SW,
    SE,
    Start,
}

impl FromStr for Maze {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut size = Coord::zero();
        let pipes = s
            .lines()
            .enumerate()
            .flat_map(|(y, line)| {
                line.chars()
                    .enumerate()
                    .map(move |(x, c)| (Coord::new(x as i32, y as i32), c))
                    .filter_map(|(coord, c)| Some((coord, Pipe::try_from(c).ok()?)))
                    .map(move |(coord, pipe)| (coord, pipe))
            })
            .inspect(|(c, _)| size = size.max(c))
            .collect::<HashMap<_, _>>();
        let start = pipes
            .iter()
            .find(|(_, &pipe)| pipe == Pipe::Start)
            .ok_or(anyhow!("Input does not contain any start"))?
            .0
            .clone();

        Ok(Self {
            pipes,
            size,
            start,
            path: Vec::new(),
        })
    }
}

impl TryFrom<char> for Pipe {
    type Error = anyhow::Error;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '|' => Ok(Self::NS),
            '-' => Ok(Self::EW),
            'J' => Ok(Self::NW),
            'L' => Ok(Self::NE),
            '7' => Ok(Self::SW),
            'F' => Ok(Self::SE),
            'S' => Ok(Self::Start),
            c => Err(anyhow!("Unknown pipe char: {c}")),
        }
    }
}

impl From<&Pipe> for char {
    fn from(pipe: &Pipe) -> Self {
        match pipe {
            Pipe::EW => '─',
            Pipe::NS => '│',
            Pipe::SE => '╭',
            Pipe::SW => '╮',
            Pipe::NW => '╯',
            Pipe::NE => '╰',
            Pipe::Start => '┼',
        }
    }
}

impl Debug for Maze {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path = self.path.iter().collect::<HashSet<_>>();
        for y in 0..=self.size.y {
            for x in 0..=self.size.x {
                let c = Coord::new(x, y);
                let sym = self.pipes.get(&c).map(char::from).unwrap_or('·');
                if path.contains(&c) {
                    write!(f, "{}{sym}{}", Fg(Red), Fg(Reset))?;
                } else {
                    write!(f, "{sym}")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Debug for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Up => write!(f, "↑"),
            Self::Right => write!(f, "→"),
            Self::Left => write!(f, "←"),
            Self::Down => write!(f, "↓"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case(include_str!("../../sample/tenth-a.txt"), 4)]
    #[case(include_str!("../../sample/tenth-b.txt"), 8)]
    fn sample_a(#[case] s: &str, #[case] expected_distance: usize) {
        let maze = Maze::from_str(s).expect("parsing");
        println!("{maze:?}");
        let path = maze
            .follow(&maze.start, Direction::Right)
            .take_while(|c| *c != maze.start)
            .collect::<Vec<_>>();
        assert_eq!(expected_distance, (path.len() + 1) / 2);
    }
}
