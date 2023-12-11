#![feature(generators, iter_from_generator)]

use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Debug,
    iter,
    ops::Add,
    str::FromStr,
};

use anyhow::anyhow;
use aoc23::Part;
use enum_iterator::{all, next_cycle, previous_cycle, Sequence};
use itertools::Itertools;
use termion::color::{Fg, LightYellow, Red, Reset, Rgb};

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

    /// Invert the "inside" of the search
    #[clap(long)]
    invert: bool,
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
                .collect();
            maze.path.len() / 2
        }
        Part::Two => {
            maze.path = maze
                .follow(&maze.start, Direction::Right)
                .take_while_inclusive(|c| *c != maze.start)
                .collect();
            maze.calculate_inside(args.invert);
            maze.inside.len()
        }
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

    fn unconnected(&self, d: Direction, ccw: bool) -> Vec<Direction> {
        // TODO: also handle ccw
        match (d, *self, ccw) {
            (_, Self::Start, _) => vec![],
            (Direction::Up, Self::NS, _)
            | (Direction::Down, Self::NS, _)
            | (Direction::Left, Self::EW, _)
            | (Direction::Right, Self::EW, _) => vec![if ccw { d.ccw() } else { d.cw() }],

            (Direction::Down, Self::NW, false) | (Direction::Right, Self::NW, true) => vec![],
            (Direction::Down, Self::NW, true) | (Direction::Right, Self::NW, false) => {
                vec![Direction::Right, Direction::Down]
            }

            (Direction::Down, Self::NE, true) | (Direction::Left, Self::NE, false) => vec![],
            (Direction::Down, Self::NE, false) | (Direction::Left, Self::NE, true) => {
                vec![Direction::Down, Direction::Left]
            }

            (Direction::Up, Self::SW, true) | (Direction::Right, Self::SW, false) => vec![],
            (Direction::Up, Self::SW, false) | (Direction::Right, Self::SW, true) => {
                vec![Direction::Right, Direction::Up]
            }

            (Direction::Up, Self::SE, false) | (Direction::Left, Self::SE, true) => vec![],
            (Direction::Up, Self::SE, true) | (Direction::Left, Self::SE, false) => {
                vec![Direction::Up, Direction::Left]
            }

            (d, p, _) => panic!("Unsupported, cannot go {d:?} within pipe {p:?}"),
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
    inside: HashSet<Coord>,
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

    fn calculate_inside(&mut self, ccw: bool) {
        let mut d = Direction::Right;
        let pathset = self.path.iter().collect::<HashSet<_>>();

        // Find all neighbors on one side (cw or ccw) of the path
        let mut queue = VecDeque::new();
        for c in &self.path {
            let pipe = self.pipes.get(c).unwrap();
            let neighbors = pipe.unconnected(d, ccw);
            for n in neighbors
                .into_iter()
                .map(|dir| c + dir)
                .filter(|n| !pathset.contains(n))
            {
                queue.push_back(n);
            }
            d = pipe.follow(d).unwrap();
        }

        // Bucket fill / region growing
        while let Some(item) = queue.pop_front() {
            self.inside.insert(item.clone());
            queue.extend(
                all::<Direction>()
                    .map(|d| &item + d)
                    .filter(|c| !pathset.contains(c) && !self.inside.contains(c)),
            );
        }
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
            inside: HashSet::new(),
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
                } else if self.inside.contains(&c) {
                    write!(f, "{}{sym}{}", Fg(LightYellow), Fg(Reset))?;
                } else {
                    write!(f, "{}{sym}{}", Fg(Rgb(100, 100, 100)), Fg(Reset))?;
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
        let mut maze = Maze::from_str(s).expect("parsing");
        maze.path = maze
            .follow(&maze.start, Direction::Right)
            .take_while_inclusive(|c| *c != maze.start)
            .collect();
        println!("{maze:?}");
        assert_eq!(expected_distance, maze.path.len() / 2);
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
        maze.path = maze
            .follow(&maze.start, Direction::Right)
            .take_while_inclusive(|c| *c != maze.start)
            .collect();
        maze.calculate_inside(ccw);
        println!("{maze:?}");
        assert_eq!(expected_inside_area, maze.inside.len());
    }
}
