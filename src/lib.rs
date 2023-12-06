pub mod fifth;
pub mod second;

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

#[derive(Resource)]
pub struct Tick(Timer);

#[derive(Default, Resource)]
pub struct Running(bool);

impl Running {
    pub fn inner(&self) -> bool {
        self.0
    }
}

impl Tick {
    pub fn new(frequency: f32) -> Self {
        Self(Timer::from_seconds(1. / frequency, TimerMode::Repeating))
    }

    pub fn inner(&mut self) -> &mut Timer {
        &mut self.0
    }
}

impl AsRef<Timer> for Tick {
    fn as_ref(&self) -> &Timer {
        &self.0
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
