use bevy::prelude::*;

use crate::{
    coord2vec, frequency_increaser, lerprgb, mouse, toggle_running, Running, Scroll, Tick,
};

use super::{Contraption, Mirror};

const TILE: f32 = 40.;
const COLOR_FADE_RAYS_AFTER_SECS: f32 = 4.;

pub fn run(machine: Contraption, frequency: f32) {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(machine)
        .insert_resource(Tick::new(frequency))
        .insert_resource(Running::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update,
                mouse,
                toggle_running,
                frequency_increaser,
                draw_beams,
            ),
        )
        .run()
}

fn setup(mut cmd: Commands, machine: Res<Contraption>) {
    cmd.spawn(Camera2dBundle {
        transform: Transform::from_xyz(
            machine.ncols as f32 * TILE / 2.,
            -machine.nrows as f32 * TILE / 2.,
            10.,
        ),
        ..default()
    })
    .insert(Scroll(1.7));
    for (coord, mirror) in machine.mirrors() {
        cmd.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::GRAY,
                custom_size: Some(Vec2::new(0.9 * TILE, 0.2 * TILE)),
                ..default()
            },
            transform: Transform::from_xyz(TILE * coord.x as f32, -TILE * coord.y as f32, 1.)
                .with_rotation(Quat::from_rotation_z(
                    match mirror {
                        Mirror::Slash => 45f32,
                        Mirror::Backslash => -45f32,
                        Mirror::SplitterLR => 0f32,
                        Mirror::SplitterUD => 90f32,
                    }
                    .to_radians(),
                )),
            ..default()
        });
    }
}

fn draw_beams(machine: Res<Contraption>, mut gizmos: Gizmos, time: Res<Time>) {
    for beam in machine.beams() {
        gizmos.linestrip_gradient_2d(beam.rays().map(|ray| {
            (
                coord2vec(ray.coord) * TILE,
                lerprgb(
                    beam.color,
                    Color::WHITE.with_a(0.75),
                    ((time.elapsed_seconds() - ray.stamp) / COLOR_FADE_RAYS_AFTER_SECS)
                        .clamp(0., 1.),
                ),
            )
        }));
    }
}

fn update(
    keys: Res<Input<KeyCode>>,
    running: Res<Running>,
    time: Res<Time>,
    mut timer: ResMut<Tick>,
    mut exit: ResMut<Events<bevy::app::AppExit>>,
    mut machine: ResMut<Contraption>,
) {
    if keys.just_pressed(KeyCode::Q) {
        exit.send(bevy::app::AppExit);
    }

    let trigger = keys.just_released(KeyCode::Tab)
        || running.inner() && timer.inner().tick(time.delta()).just_finished();

    if !trigger {
        return;
    }

    if !machine.is_in_equilibrium() {
        machine.advance(time.elapsed_seconds());
    }
}
