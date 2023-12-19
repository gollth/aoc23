pub mod animation;

use anyhow::anyhow;
use bevy::ecs::system::Resource;
use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    ops::Not,
    str::FromStr,
};
use termion::color::{Fg, Reset, Rgb, Yellow};

use crate::Coord;

pub const NORTH: Coord = Coord::new(0, -1);
pub const SOUTH: Coord = Coord::new(0, 1);
pub const EAST: Coord = Coord::new(1, 0);
pub const WEST: Coord = Coord::new(-1, 0);

pub const CYCLE: [Coord; 4] = [NORTH, WEST, SOUTH, EAST];

#[derive(Debug, Clone, Resource)]
pub struct Platform {
    rocks: HashMap<Coord, Rock>,
    nrows: i32,
    ncols: i32,
}

impl PartialEq for Platform {
    fn eq(&self, other: &Self) -> bool {
        self.ncols == other.ncols
            && self.nrows == other.nrows
            && self.round_rocks() == other.round_rocks()
    }
}

#[derive(Default, Debug, PartialEq, Copy, Clone, Eq)]
pub enum Rock {
    #[default]
    None,
    Round,
    Square,
}

impl Platform {
    pub(crate) fn get(&self, c: Coord) -> Rock {
        if c.x < 0 || self.ncols <= c.x || c.y < 0 || self.nrows <= c.y {
            return Rock::Square;
        }
        self.rocks.get(&c).copied().unwrap_or_default()
    }

    fn outer(&self, dir: Coord) -> i32 {
        if dir == NORTH || dir == SOUTH {
            return self.ncols;
        }
        if dir == EAST || dir == WEST {
            return self.nrows;
        }
        panic!("Only N,S,W or E directions supported")
    }

    fn inner_iter(&self, dir: Coord) -> Box<dyn Iterator<Item = i32>> {
        if dir == NORTH {
            Box::new(-1..=self.nrows)
        } else if dir == SOUTH {
            Box::new((-1..=self.nrows).rev())
        } else if dir == EAST {
            Box::new((-1..=self.ncols).rev())
        } else if dir == WEST {
            Box::new(-1..=self.ncols)
        } else {
            panic!("Only N,S,W or E directions supported")
        }
    }

    fn coord(&self, dir: Coord, outer: i32, inner: i32) -> Coord {
        if dir == NORTH || dir == SOUTH {
            Coord::new(outer, inner)
        } else if dir == EAST || dir == WEST {
            Coord::new(inner, outer)
        } else {
            panic!("Only N,S,W or E directions supported")
        }
    }

    pub fn tilt(&mut self, dir: Coord) {
        let mut rocks = HashMap::new();
        for outer in 0..self.outer(dir) {
            let new_coords = self
                .inner_iter(dir)
                .map(|inner| self.coord(dir, outer, inner))
                .map(|c| (c, self.get(c)))
                .group_by(|(_, r)| r == &Rock::Square)
                .into_iter()
                .filter_map(|(is_square, region)| is_square.not().then_some(region))
                .filter_map(|region| {
                    let mut region = region.peekable();
                    region.peek().copied().map(|(start, _)| {
                        (
                            start,
                            region.filter(|(_, rock)| rock == &Rock::Round).count(),
                        )
                    })
                })
                .filter(|(_, n)| *n > 0)
                .flat_map(move |(start, n)| (0..).map(move |i| start - dir * i).take(n))
                .map(|coord| (coord, Rock::Round))
                .collect::<HashMap<_, _>>();
            rocks.extend(new_coords);
        }
        self.rocks.retain(|_, rock| rock != &Rock::Round);
        self.rocks.extend(rocks);
    }

    pub fn total_north_load(&self) -> i32 {
        self.rocks
            .iter()
            .filter(|(_, item)| item == &&Rock::Round)
            .map(|(coord, _)| self.nrows - coord.y)
            .sum()
    }
    pub fn round_rocks(&self) -> HashSet<Coord> {
        self.rocks
            .iter()
            .filter(|(_, rock)| rock == &&Rock::Round)
            .map(|(coord, _)| coord)
            .copied()
            .collect()
    }
}

impl FromStr for Platform {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let rocks = s
            .lines()
            .enumerate()
            .flat_map(|(y, line)| {
                line.trim().chars().enumerate().map(move |(x, c)| {
                    Ok::<(Coord, Rock), anyhow::Error>((
                        Coord::new(x as i32, y as i32),
                        Rock::try_from(c)?,
                    ))
                })
            })
            .process_results(|iter| iter.collect::<HashMap<_, _>>())?;
        if rocks.is_empty() {
            return Err(anyhow!("Empty platforms not allowed"));
        }
        let ncols = rocks.keys().map(|i| i.x).max().unwrap_or_default() + 1;
        let nrows = rocks.keys().map(|i| i.y).max().unwrap_or_default() + 1;
        Ok(Self {
            rocks,
            ncols,
            nrows,
        })
    }
}

impl Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "╭")?;
        for _ in 0..self.ncols + 2 {
            write!(f, "─")?;
        }
        writeln!(f, "╮")?;
        for y in -1..=self.nrows {
            write!(f, "│")?;
            for x in -1..=self.ncols {
                let coord = Coord::new(x, y);
                let rock = self.get(coord);
                if rock == Rock::Square {
                    write!(f, "{}", Fg(Rgb(160, 160, 160)))?;
                } else if rock == Rock::Round {
                    write!(f, "{}", Fg(Yellow))?;
                }
                write!(f, "{}", rock)?;
                write!(f, "{}", Fg(Reset))?;
            }
            writeln!(f, "│")?;
        }
        write!(f, "╰")?;
        for _ in 0..self.ncols + 2 {
            write!(f, "─")?;
        }
        write!(f, "╯")?;
        Ok(())
    }
}

impl TryFrom<char> for Rock {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(Rock::None),
            'O' => Ok(Rock::Round),
            '#' => Ok(Rock::Square),
            _ => Err(anyhow!("Unknown rock: {value}")),
        }
    }
}
impl Display for Rock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => '·',
                Self::Round => '●',
                Self::Square => '▧',
            }
        )
    }
}
