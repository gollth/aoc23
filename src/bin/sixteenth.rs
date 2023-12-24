#![feature(let_chains)]

use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::{Debug, Display},
    iter::once,
    str::FromStr,
    thread::sleep,
    time::Duration,
};

use anyhow::anyhow;
use aoc23::{lerphsl, Coord, Direction, Part};
use bevy::render::color::Color;
use clap::Parser;
use enum_iterator::all;
use termion::color::{Fg, Reset, Rgb};

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

    #[clap(long, short, default_value_t = 2.)]
    frequency: f32,
}

fn main() -> anyhow::Result<()> {
    let args = Options::parse();
    let input = std::fs::read_to_string(args.input)?;

    let mut contraption = Contraption::from_str(&input)?;

    while !contraption.is_in_equilibrium() {
        contraption.advance();
        if args.animate && args.frequency > 0. {
            print!("\x1B[2J\x1B[1;1H");
            println!("{contraption:?}");
            sleep(Duration::from_secs_f32(1. / args.frequency));
        }
    }

    if args.animate && args.frequency == 0. {
        println!("{contraption:?}");
    }

    let solution = contraption.energized_cells().len();
    println!("Solution: {solution}");

    Ok(())
}

struct Contraption {
    cells: HashMap<Coord, Mirror>,
    nrows: i32,
    ncols: i32,
    active: VecDeque<Beam>,
    closed: Vec<Beam>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct Ray {
    coord: Coord,
    direction: Direction,
}

#[derive(Debug)]
struct Beam {
    latest: Ray,
    rays: HashSet<Ray>,
    color: Color,
    nrows: i32,
    ncols: i32,
}

impl Ray {
    fn new(coord: Coord, direction: Direction) -> Self {
        Self { coord, direction }
    }

    fn cast(&self) -> Self {
        Self {
            coord: self.coord + Coord::from(self.direction),
            direction: self.direction,
        }
    }

    fn cw(&self) -> Self {
        let mut other = self.clone();
        other.direction = self.direction.cw();
        other
    }
    fn ccw(&self) -> Self {
        let mut other = self.clone();
        other.direction = self.direction.ccw();
        other
    }

    fn is_out_of_bounds(&self, ncols: i32, nrows: i32) -> bool {
        self.coord != Coord::new(0, 0)
            && (self.coord.x < 0
                || ncols <= self.coord.x
                || self.coord.y < 0
                || nrows <= self.coord.y)
    }
}
impl Beam {
    fn new(x: i32, y: i32, direction: Direction, hue: f32, ncols: i32, nrows: i32) -> Self {
        let latest = Ray {
            coord: Coord::new(x, y),
            direction,
        };
        let rays = HashSet::default();
        let color = Color::hsl(hue, 1., 0.5);
        Self {
            rays,
            latest,
            color,
            nrows,
            ncols,
        }
    }

    fn is_finished<'a>(&self, mut beams: impl Iterator<Item = &'a HashSet<Ray>>) -> bool {
        beams.any(|beam| beam.contains(&self.latest))
            || self.latest.is_out_of_bounds(self.ncols, self.nrows)
    }

    fn advance(&mut self, cells: &HashMap<Coord, Mirror>) -> Option<Beam> {
        use Direction::{Down, Left, Right, Up};
        self.rays.insert(self.latest.clone());
        let (new_beam, next) = match (cells.get(&self.latest.coord), self.latest.direction) {
            (None, _) => (None, self.latest.cast()), // empty space, simply cast the ray forward
            (Some(Mirror::Slash), Right | Left) => (None, self.latest.ccw().cast()),
            (Some(Mirror::Slash), Up | Down) => (None, self.latest.cw().cast()),
            (Some(Mirror::Backslash), Right | Left) => (None, self.latest.cw().cast()),
            (Some(Mirror::Backslash), Up | Down) => (None, self.latest.ccw().cast()),
            (Some(Mirror::SplitterUD), Up | Down) => (None, self.latest.cast()),
            (Some(Mirror::SplitterLR), Left | Right) => (None, self.latest.cast()),
            (Some(Mirror::SplitterUD), Left | Right) | (Some(Mirror::SplitterLR), Up | Down) => {
                let other = self.latest.cw();
                let me = self.latest.ccw();
                (
                    Some(Beam::new(
                        other.coord.x,
                        other.coord.y,
                        other.direction,
                        (self.color.h() + 45.) % 360.,
                        self.ncols,
                        self.nrows,
                    )),
                    me,
                )
            }
        };
        self.latest = next;
        new_beam
    }
}

impl Contraption {
    fn energized_cells(&self) -> HashSet<Coord> {
        self.closed
            .iter()
            .flat_map(|beam| beam.rays.iter().map(|ray| ray.coord))
            .collect()
    }

    fn is_in_equilibrium(&self) -> bool {
        self.active.is_empty()
    }
    fn rays_iter(&self) -> impl Iterator<Item = &HashSet<Ray>> {
        self.active
            .iter()
            .chain(self.closed.iter())
            .map(|beam| &beam.rays)
    }
    fn advance(&mut self) {
        let mut n = self.active.len();
        while n > 0 && let Some(mut beam) = self.active.pop_front() {
            n -= 1;
            if beam.is_finished(self.rays_iter().chain(once(&beam.rays))) {
                self.closed.push(beam);
                continue;
            }
            if let Some(new_beam) = beam.advance(&self.cells) {
                self.active.push_back(new_beam);
            }
            self.active.push_back(beam);
        }
    }
}

impl FromStr for Contraption {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cells = s
            .lines()
            .enumerate()
            .flat_map(|(y, line)| {
                line.trim()
                    .chars()
                    .enumerate()
                    .filter(|(_, c)| *c != '.')
                    .map(move |(x, c)| {
                        (Coord::new(x as i32, y as i32), Mirror::try_from(c).unwrap())
                    })
            })
            .collect::<HashMap<_, _>>();
        let nrows = s.lines().count() as i32;
        let ncols = s
            .lines()
            .next()
            .ok_or(anyhow!("Contraption must contain at least one line"))?
            .trim()
            .chars()
            .count() as i32;
        Ok(Self {
            cells,
            ncols,
            nrows,
            active: [Beam::new(0, 0, Direction::Right, 0., ncols, nrows)]
                .into_iter()
                .collect(),
            closed: Vec::new(),
        })
    }
}

impl Debug for Contraption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let reset = Fg(Reset);
        write!(f, "╭")?;
        for _ in 0..self.ncols {
            write!(f, "─")?;
        }
        writeln!(f, "╮")?;
        for y in 0..self.nrows {
            write!(f, "│")?;
            for x in 0..self.ncols {
                let coord = Coord::new(x, y);
                let color = self
                    .active
                    .iter()
                    .chain(self.closed.iter())
                    .filter(|beam| {
                        all::<Direction>().any(|dir| beam.rays.contains(&Ray::new(coord, dir)))
                    })
                    .map(|beam| beam.color)
                    .reduce(|a, b| lerphsl(a, b, 0.5))
                    .unwrap_or(Color::GRAY);
                let color = color.as_rgba_u8();
                let fg = Fg(Rgb(color[0], color[1], color[2]));
                if let Some(mirror) = self.cells.get(&coord) {
                    write!(f, "{fg}{}{reset}", mirror)?;
                } else {
                    write!(f, "{fg}·{reset}")?;
                }
            }
            writeln!(f, "│")?;
        }
        write!(f, "╰")?;
        for _ in 0..self.ncols {
            write!(f, "─")?;
        }
        write!(f, "╯")?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Mirror {
    Slash,
    Backslash,
    SplitterLR,
    SplitterUD,
}

impl Display for Mirror {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Backslash => write!(f, "⟍"),
            Self::Slash => write!(f, "⟋"),
            Self::SplitterLR => write!(f, "―"),
            Self::SplitterUD => write!(f, "|"),
        }
    }
}

impl TryFrom<char> for Mirror {
    type Error = anyhow::Error;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '-' => Ok(Mirror::SplitterLR),
            '|' => Ok(Mirror::SplitterUD),
            '/' => Ok(Mirror::Slash),
            '\\' => Ok(Mirror::Backslash),
            _ => Err(anyhow!("Unknown mirror character: {value}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(46, include_str!("../../sample/sixteenth.txt"))]
    #[case(
        9,
        "..|..
         .....
         ..-.."
    )]
    #[case(
        18,
        r#"...\...
           .......
           -......
           .......
           \../..."#
    )]
    #[case(
        16,
        "|....-
         ......
         ......
         -....|"
    )]
    #[case(
        41,
        r#"......|...\..\...
           ..../........|...
           ....\.-.../......
           ......|....../...
           ................."#
    )]
    fn sample_a(#[case] expectation: usize, #[case] input: &str) {
        let mut max_steps = 100;
        let mut contraption = Contraption::from_str(input).expect("parsing");
        println!(
            "Contraption Size: {}/{}",
            contraption.ncols, contraption.nrows
        );
        while !contraption.is_in_equilibrium() {
            contraption.advance();
            println!("{contraption:?}");
            println!(
                "Beams: {:?}",
                contraption
                    .active
                    .iter()
                    .map(|beam| (
                        beam.latest.direction,
                        beam.latest.coord.x,
                        beam.latest.coord.y
                    ))
                    .collect::<Vec<_>>()
            );
            if max_steps == 0 {
                panic!("Reached max steps, propably infinite loop");
            }
            max_steps -= 1;
        }
        assert_eq!(expectation, contraption.energized_cells().len())
    }
}
