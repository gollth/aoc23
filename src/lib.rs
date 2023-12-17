#![feature(generators, iter_from_generator, iter_intersperse)]

pub mod fifth;
pub mod second;
pub mod ten;
pub mod thirteenth;

use anyhow::anyhow;
use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};
use clap::ValueEnum;
use std::convert::AsRef;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, ValueEnum)]
pub enum Part {
    One,
    Two,
}

pub fn anyhowing(e: nom::error::Error<&str>) -> anyhow::Error {
    anyhow!("{e}")
}

pub(crate) fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
pub(crate) fn lerpc(a: Color, b: Color, t: f32) -> Color {
    Color::rgba(
        lerp(a.r(), b.r(), t),
        lerp(a.g(), b.g(), t),
        lerp(a.b(), b.b(), t),
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
const ZOOM_SENSITIVITY: f32 = 0.5;

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
