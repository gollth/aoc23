#![feature(
    generators,
    iter_from_generator,
    iter_intersperse,
    let_chains,
    iter_array_chunks
)]

pub mod fifteenth;
pub mod fifth;
pub mod fourteenth;
pub mod second;
pub mod sixteenth;
pub mod ten;
pub mod thirteenth;

use anyhow::anyhow;
use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use clap::ValueEnum;
use enum_iterator::{next_cycle, previous_cycle, Sequence};
use std::{convert::AsRef, fmt::Debug};

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, ValueEnum)]
pub enum Part {
    #[default]
    One,
    Two,
}

pub type Coord = euclid::Vector2D<i32, euclid::UnknownUnit>;

pub fn coord2vec(coord: Coord) -> Vec2 {
    Vec2::new(coord.x as f32, -coord.y as f32)
}

pub fn anyhowing(e: nom::error::Error<&str>) -> anyhow::Error {
    anyhow!("{e}")
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Sequence)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    pub fn cw(&self) -> Self {
        next_cycle(self).unwrap()
    }
    pub fn ccw(&self) -> Self {
        previous_cycle(self).unwrap()
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
impl From<Direction> for Coord {
    fn from(dir: Direction) -> Self {
        match dir {
            Direction::Up => Coord::new(0, -1),
            Direction::Down => Coord::new(0, 1),
            Direction::Left => Coord::new(-1, 0),
            Direction::Right => Coord::new(1, 0),
        }
    }
}

pub(crate) fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
pub(crate) fn lerprgb(a: Color, b: Color, t: f32) -> Color {
    Color::rgba(
        lerp(a.r(), b.r(), t),
        lerp(a.g(), b.g(), t),
        lerp(a.b(), b.b(), t),
        lerp(a.a(), b.a(), t),
    )
}
pub fn lerphsl(a: Color, b: Color, t: f32) -> Color {
    Color::hsla(
        lerp(a.h(), b.h(), t),
        lerp(a.s(), b.s(), t),
        lerp(a.l(), b.l(), t),
        lerp(a.a(), b.a(), t),
    )
}

#[derive(Resource)]
pub struct Tick {
    timer: Timer,
    f: f32,
}

#[derive(Default, Resource, Debug)]
pub struct Running(bool);

impl Running {
    pub fn inner(&self) -> bool {
        self.0
    }
}

impl Tick {
    pub fn new(f: f32) -> Self {
        Self {
            timer: Timer::from_seconds(1. / f, TimerMode::Repeating),
            f,
        }
    }

    pub fn inner(&mut self) -> &mut Timer {
        &mut self.timer
    }

    pub fn frequency(&self) -> f32 {
        self.f
    }
    pub fn set_frequency(&mut self, f: f32) {
        self.timer = Timer::from_seconds(1. / f, TimerMode::Repeating);
        self.f = f;
    }
}

impl AsRef<Timer> for Tick {
    fn as_ref(&self) -> &Timer {
        &self.timer
    }
}

pub fn frequency_increaser(keys: Res<Input<KeyCode>>, mut timer: ResMut<Tick>) {
    let f = timer.frequency();
    if keys.just_released(KeyCode::J) {
        timer.set_frequency(f * 2.);
    }
    if keys.just_released(KeyCode::K) {
        timer.set_frequency(f / 2.);
    }
}

#[derive(Debug, Component)]
pub struct Scroll(pub f32);

const ZOOM_SPEED: f32 = 4.0;

const ZOOM_SENSITIVITY: f32 = 0.1;
pub fn mouse(
    time: Res<Time>,
    mouse: Res<Input<MouseButton>>,
    mut motion: EventReader<MouseMotion>,
    mut scroll: EventReader<MouseWheel>,
    mut query: Query<(&mut Scroll, &mut Transform), With<Camera>>,
) {
    let pressed = mouse.any_pressed([MouseButton::Left, MouseButton::Right]);
    let motion = motion.read().map(|ev| ev.delta).sum::<Vec2>();
    let delta = scroll.read().map(|ev| ev.y).sum::<f32>();

    for (mut scroll, mut tf) in query.iter_mut() {
        scroll.0 += delta * ZOOM_SENSITIVITY;
        let mut s = tf.scale.x;
        s += ZOOM_SPEED * (scroll.0.exp() - s) * time.delta_seconds();
        tf.scale = Vec3::splat(s);
        if pressed {
            tf.translation += Vec3::new(-motion.x, motion.y, 0.) * s;
        }
    }
}

pub fn toggle_running(keys: Res<Input<KeyCode>>, mut run: ResMut<Running>) {
    if keys.just_released(KeyCode::Space) {
        run.0 ^= true;
    }
}

pub(crate) fn rect(x: f32, y: f32, z: f32, w: f32, h: f32, color: Color) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            color,
            custom_size: Some(Vec2::new(w, h)),
            ..default()
        },
        transform: Transform::from_xyz(x, y, z),
        ..default()
    }
}

pub(crate) fn arc_segment(n: usize, arc: &ArcSegment) -> Mesh {
    let mut vertices = Vec::new();
    let mut faces = Vec::new();

    for i in 0..n {
        let t = arc.phi + arc.alpha * (i as f32 / (n - 1) as f32);
        let (x, y) = t.sin_cos();
        vertices.push([arc.ro * x, arc.ro * y, 0.]);
        vertices.push([arc.ri * x, arc.ri * y, 0.]);
    }

    for i in (0..2 * n as u32).step_by(2) {
        faces.extend_from_slice(&[i, i + 1, i + 3]);
        faces.extend_from_slice(&[i, i + 3, i + 2]);
    }

    Mesh::new(PrimitiveTopology::TriangleList)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
        .with_indices(Some(Indices::U32(faces)))
}

#[derive(Default, Debug, Component, Clone, PartialEq)]
pub(crate) struct ArcSegment {
    /// Offset
    phi: f32,
    /// Length
    alpha: f32,
    /// Inner radius
    ri: f32,
    /// Outer radius
    ro: f32,
}

pub(crate) fn in_states<S>(states: &'static [S]) -> impl Condition<()>
where
    S: States,
{
    IntoSystem::into_system(|current_state: Res<State<S>>| {
        states.iter().any(|s| s == current_state.get())
    })
}

pub fn cycle<T, I>(mut xs: I) -> Option<(usize, usize)>
where
    T: PartialEq,
    I: Iterator<Item = T> + Clone,
{
    // Let hare run twice as fast as tortoise until they meet
    let mut tortoise = xs.clone();
    let mut hare = xs.clone();
    hare.next();
    while tortoise.next()? != hare.next()? {
        hare.next()?;
    }

    // Reset tortoise to beginning at let both run in same speed until they meet, to find offset (mu)
    let mut mu = 0;
    let mut tortoise = xs.clone();
    while tortoise.next()? != hare.next()? {
        mu += 1;
    }

    // Let hare run one step at a time and tortoise rest to find length of cycle (lambda)
    let mut lambda = 1;
    let x_mu = xs.nth(mu)?;

    while hare.next()? != x_mu {
        lambda += 1
    }

    Some((mu, lambda))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::iter::{empty, once};

    #[rstest]
    #[case(None, empty())]
    #[case(None, 1..6)]
    #[case(Some((0, 3)), (1..=3).cycle())]
    #[case(Some((1, 3)), once(17).chain((1..=3).cycle()))]
    #[case(Some((5, 6)), (42..=46).chain((1..=6).cycle()))]
    fn tortoise_hare(
        #[case] expected: Option<(usize, usize)>,
        #[case] xs: impl Iterator<Item = i32> + Clone,
    ) {
        assert_eq!(expected, cycle(xs));
    }
}
