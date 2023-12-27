use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::{Debug, Display},
    iter::once,
    str::FromStr,
};

use anyhow::anyhow;
use bevy::{ecs::system::Resource, render::color::Color};
use enum_iterator::all;
use rand::{thread_rng, Rng};
use termion::color::{Fg, Reset, Rgb};

use crate::{lerphsl, Coord, Direction};

pub mod animation;

pub const PART_ONE_ENTRY: (Direction, i32) = (Direction::Right, 0);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mirror {
    Slash,
    Backslash,
    SplitterLR,
    SplitterUD,
}

#[derive(Resource)]
pub struct Contraption {
    cells: HashMap<Coord, Mirror>,
    nrows: i32,
    ncols: i32,
    active: VecDeque<Beam>,
    closed: Vec<Beam>,
}

#[derive(Debug, Clone)]
pub struct Ray {
    pub coord: Coord,
    pub direction: Direction,
    stamp: f32,
}

#[derive(Debug)]
pub struct Beam {
    latest: Ray,
    rays: Vec<Ray>,
    color: Color,
    nrows: i32,
    ncols: i32,
}

impl Ray {
    pub fn new(coord: Coord, direction: Direction, stamp: f32) -> Self {
        Self {
            coord,
            direction,
            stamp,
        }
    }

    pub fn cast(&self, stamp: f32) -> Self {
        Self::new(
            self.coord + Coord::from(self.direction),
            self.direction,
            stamp,
        )
    }

    pub fn cw(&self) -> Self {
        let mut other = self.clone();
        other.direction = self.direction.cw();
        other
    }

    pub fn ccw(&self) -> Self {
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

impl PartialEq for Ray {
    fn eq(&self, other: &Self) -> bool {
        // Skip comparaision of stamp here, since only used for visualziation
        self.coord == other.coord && self.direction == other.direction
    }
}

impl Beam {
    fn new(ray: Ray, hue: f32, ncols: i32, nrows: i32) -> Self {
        let rays = Vec::default();
        let color = Color::hsl(hue, 1., 0.5);
        Self {
            rays,
            latest: ray,
            color,
            nrows,
            ncols,
        }
    }

    pub(crate) fn rays(&self) -> impl Iterator<Item = &Ray> {
        self.rays.iter()
    }

    pub fn tip(&self) -> &Ray {
        &self.latest
    }

    fn is_finished<'a>(&self, mut beams: impl Iterator<Item = &'a [Ray]>) -> bool {
        beams.any(|beam| beam.contains(&self.latest))
            || self.latest.is_out_of_bounds(self.ncols, self.nrows)
    }

    fn advance(&mut self, cells: &HashMap<Coord, Mirror>, stamp: f32) -> Option<Beam> {
        self.rays.push(self.latest.clone());
        use Direction::{Down, Left, Right, Up};
        let (new_beam, next) = match (cells.get(&self.latest.coord), self.latest.direction) {
            (None, _) => (None, self.latest.cast(stamp)), // empty space, simply cast the ray forward
            (Some(Mirror::Slash), Right | Left) => (None, self.latest.ccw().cast(stamp)),
            (Some(Mirror::Slash), Up | Down) => (None, self.latest.cw().cast(stamp)),
            (Some(Mirror::Backslash), Right | Left) => (None, self.latest.cw().cast(stamp)),
            (Some(Mirror::Backslash), Up | Down) => (None, self.latest.ccw().cast(stamp)),
            (Some(Mirror::SplitterUD), Up | Down) => (None, self.latest.cast(stamp)),
            (Some(Mirror::SplitterLR), Left | Right) => (None, self.latest.cast(stamp)),
            (Some(Mirror::SplitterUD), Left | Right) | (Some(Mirror::SplitterLR), Up | Down) => {
                let other = self.latest.cw();
                let me = self.latest.ccw();
                (
                    Some(Beam::new(
                        other,
                        (self.color.h() + thread_rng().gen_range(90.0..270.0)) % 360.,
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
    pub fn ncols(&self) -> i32 {
        self.ncols
    }

    pub fn nrows(&self) -> i32 {
        self.nrows
    }

    pub fn reset(&mut self) {
        self.active.clear();
        self.closed.clear();
    }

    pub fn set_entry(&mut self, (dir, i): (Direction, i32)) -> anyhow::Result<()> {
        if !self.active.is_empty() {
            return Err(anyhow!(
                "Setting a new entry is only allowed before the contraption ever advanced"
            ));
        }

        let ray = Ray::new(Coord::from(dir.cw()).abs() * i, dir, 0.);
        self.active = [Beam::new(ray, 0., self.ncols, self.nrows)]
            .into_iter()
            .collect();
        Ok(())
    }

    pub fn energized_cells(&self) -> HashSet<Coord> {
        self.closed
            .iter()
            .flat_map(|beam| beam.rays.iter().map(|ray| ray.coord))
            .collect()
    }

    pub fn is_in_equilibrium(&self) -> bool {
        self.active.is_empty()
    }

    pub fn mirrors(&self) -> impl Iterator<Item = (&Coord, &Mirror)> {
        self.cells.iter()
    }

    fn rays_iter(&self) -> impl Iterator<Item = &[Ray]> {
        self.active
            .iter()
            .chain(self.closed.iter())
            .map(|beam| beam.rays.as_slice())
    }

    pub(crate) fn beams(&self) -> impl Iterator<Item = &Beam> {
        self.active.iter().chain(self.closed.iter())
    }
    pub fn active_beams(&self) -> impl Iterator<Item = &Beam> {
        self.active.iter()
    }

    pub fn advance(&mut self, stamp: f32) {
        let mut n = self.active.len();
        while n > 0 && let Some(mut beam) = self.active.pop_front() {
            n -= 1;
            if beam.is_finished(self.rays_iter().chain(once(beam.rays.as_slice()))) {
                self.closed.push(beam);
                continue;
            }
            if let Some(new_beam) = beam.advance(&self.cells, stamp) {
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
            active: VecDeque::new(),
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
                    .beams()
                    .filter(|beam| {
                        all::<Direction>()
                            .any(|dir| beam.rays.contains(&Ray::new(coord, dir, f32::NAN)))
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
