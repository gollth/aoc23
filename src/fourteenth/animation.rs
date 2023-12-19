use bevy::{
    prelude::*,
    sprite::{Anchor, MaterialMesh2dBundle},
};
use bevy_rapier2d::prelude::*;
use enum_iterator::{next_cycle, Sequence};
use itertools::Itertools;
use lazy_static::lazy_static;

use crate::{in_states, lerp, mouse, rect, Coord, Scroll};

use super::{Platform, Rock};

const SIZE: f32 = 100.;
const GAP: f32 = 0.01 * SIZE;
const SETTLED_THRESHOLD: f32 = 0.005 * SIZE;
const STIFFNESS: f32 = 5000.;
const DAMPING: f32 = 5.;
const FONT_SIZE: f32 = 40.;

lazy_static! {
    static ref STYLE: TextStyle = TextStyle {
        font_size: FONT_SIZE,
        color: Color::WHITE,
        ..default()
    };
}

pub fn run(platform: Platform, max_load: f32) {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.))
        // .add_plugins(RapierDebugRenderPlugin::default())
        .insert_resource(platform)
        .insert_resource(TotalLoad::default())
        .insert_resource(MaxLoad(max_load))
        .add_state::<Tilt>()
        .add_state::<Motion>()
        .add_state::<Simulation>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update,
                mouse,
                stress_test_n,
                stress_test_s,
                stress_test_w,
                stress_test_e,
                track_ball_columns,
                update_total,
                detect_pause_play,
            ),
        )
        .add_systems(OnEnter(Simulation::Paused), disable_gravity)
        .add_systems(OnEnter(Simulation::Playing), enable_gravity)
        .add_systems(
            Update,
            (
                detect_settlement.run_if(in_state(Motion::Moving)),
                detect_movement.run_if(in_state(Motion::Settled)),
                stabilize_on_rows.run_if(in_states(&[Tilt::East, Tilt::West])),
                stabilize_on_colums.run_if(in_states(&[Tilt::North, Tilt::South])),
            ),
        )
        .add_systems(OnExit(Motion::Moving), change_gravity)
        .run()
}

#[derive(Debug, Component)]
struct Ball;
#[derive(Debug, Component)]
struct Support;
#[derive(Debug, Component)]
struct Total;

#[derive(Debug, Component, PartialEq, Eq)]
struct Index((i32, i32));

impl From<Vec3> for Index {
    fn from(v: Vec3) -> Self {
        Self(((v.x / SIZE).round() as i32, (v.y / SIZE).round() as i32))
    }
}

#[derive(Default, Debug, States, Hash, PartialEq, Eq, Clone)]
enum Motion {
    #[default]
    Settled,
    Moving,
}

#[derive(Default, Debug, States, Hash, PartialEq, Eq, Clone)]
enum Simulation {
    #[default]
    Paused,
    Playing,
}

#[derive(Debug, Default, Sequence, States, Hash, PartialEq, Eq, Clone, Copy)]
enum Tilt {
    #[default]
    North,
    West,
    South,
    East,
}

#[derive(Debug, Default, Resource)]
struct TotalLoad(i32);

#[derive(Debug, Default, Resource)]
struct MaxLoad(f32);

impl From<&Tilt> for Vec2 {
    fn from(d: &Tilt) -> Self {
        match d {
            Tilt::North => Vec2::Y,
            Tilt::West => Vec2::NEG_X,
            Tilt::South => Vec2::NEG_Y,
            Tilt::East => Vec2::X,
        }
    }
}

fn setup(
    mut cmd: Commands,
    platform: Res<Platform>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    cmd.spawn(Camera2dBundle {
        transform: Transform::from_xyz(
            platform.ncols as f32 * SIZE / 2.,
            platform.nrows as f32 * SIZE / 2.,
            0.,
        ),
        ..default()
    })
    .insert(Scroll(1.));

    for (x, y) in (-1..=platform.ncols).cartesian_product(-1..=platform.nrows) {
        match platform.get(Coord::new(x, platform.nrows - 1 - y)) {
            Rock::None => continue,
            Rock::Round => {
                let radius = (SIZE - GAP) / 2.;
                cmd.spawn(MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(radius).into()).into(),
                    material: materials.add(ColorMaterial::from(Color::WHITE)),
                    transform: Transform::from_xyz(x as f32 * SIZE, y as f32 * SIZE, 1.),
                    ..default()
                })
                .insert(Ball)
                .insert(GravityScale(10.))
                .insert(Collider::ball(radius))
                .insert(ExternalForce::default())
                .insert(Sleeping::disabled())
                .insert(Velocity::zero())
                .insert(LockedAxes::ROTATION_LOCKED)
                .insert(RigidBody::Dynamic)
                .with_children(|parent| {
                    parent.spawn(Text2dBundle {
                        text: Text::from_section(
                            "x",
                            TextStyle {
                                font_size: FONT_SIZE,
                                color: Color::BLACK,
                                ..default()
                            },
                        ),
                        transform: Transform::from_xyz(0., 0., 2.),
                        ..default()
                    });
                });
            }

            Rock::Square => {
                cmd.spawn(rect(
                    x as f32 * SIZE,
                    y as f32 * SIZE,
                    1.,
                    SIZE,
                    SIZE,
                    Color::DARK_GRAY,
                ))
                .insert(Collider::cuboid(SIZE / 2., SIZE / 2.))
                .insert(Index((x, y)))
                .insert(Support);
            }
        }
    }

    // North support
    for i in 0..platform.ncols {
        let position = Vec3::new(i as f32 * SIZE, platform.nrows as f32 * SIZE, 5.);
        cmd.spawn(Text2dBundle {
            text: Text::from_section("-", STYLE.clone()).with_alignment(TextAlignment::Center),
            transform: Transform::from_translation(position),
            ..default()
        })
        .insert(Index::from(position));
    }

    // South support
    for i in 0..platform.ncols {
        let position = Vec3::new(i as f32 * SIZE, -1. * SIZE, 5.);
        cmd.spawn(Text2dBundle {
            text: Text::from_section("-", STYLE.clone()).with_alignment(TextAlignment::Center),
            transform: Transform::from_translation(position),
            ..default()
        })
        .insert(Index::from(position));
    }

    // West support
    for i in 0..platform.nrows {
        let position = Vec3::new(-1. * SIZE, i as f32 * SIZE, 5.);
        cmd.spawn(Text2dBundle {
            text: Text::from_section("-", STYLE.clone()).with_alignment(TextAlignment::Center),
            transform: Transform::from_translation(position),
            ..default()
        })
        .insert(Index::from(position));
    }

    // East support
    for i in 0..platform.nrows {
        let position = Vec3::new(platform.ncols as f32 * SIZE, i as f32 * SIZE, 5.);
        cmd.spawn(Text2dBundle {
            text: Text::from_section("-", STYLE.clone()).with_alignment(TextAlignment::Center),
            transform: Transform::from_translation(position),
            ..default()
        })
        .insert(Index::from(position));
    }

    cmd.spawn(Text2dBundle {
        text: Text::from_sections(vec![
            TextSection::new(
                "Total  ",
                TextStyle {
                    font_size: 2.5 * FONT_SIZE,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            TextSection::new(
                "---",
                TextStyle {
                    font_size: 2.5 * FONT_SIZE,
                    color: Color::WHITE,
                    ..default()
                },
            ),
        ])
        .with_alignment(TextAlignment::Center),
        transform: Transform::from_xyz(
            (platform.nrows - 1) as f32 * SIZE / 2.,
            (platform.ncols + 2) as f32 * SIZE,
            0.,
        ),
        text_anchor: Anchor::Center,
        ..default()
    })
    .insert(Total);
}

fn detect_settlement(
    rigidbodies: Query<&Velocity, With<Ball>>,
    mut motion: ResMut<NextState<Motion>>,
) {
    if rigidbodies
        .iter()
        .all(|body| body.linvel.length() <= SETTLED_THRESHOLD)
    {
        motion.set(Motion::Settled);
    }
}

fn detect_movement(
    rigidbodies: Query<&Velocity, With<Ball>>,
    mut motion: ResMut<NextState<Motion>>,
) {
    if rigidbodies
        .iter()
        .any(|body| body.linvel.length() > SETTLED_THRESHOLD)
    {
        motion.set(Motion::Moving);
    }
}

fn detect_pause_play(
    keys: Res<Input<KeyCode>>,
    state: Res<State<Simulation>>,
    mut next: ResMut<NextState<Simulation>>,
) {
    if keys.just_released(KeyCode::Space) {
        if state.get() == &Simulation::Paused {
            next.set(Simulation::Playing);
        }
        if state.get() == &Simulation::Playing {
            next.set(Simulation::Paused);
        }
    }
}

fn disable_gravity(mut config: ResMut<RapierConfiguration>) {
    config.physics_pipeline_active = false;
}
fn enable_gravity(mut config: ResMut<RapierConfiguration>, state: Res<State<Tilt>>) {
    config.gravity = Vec2::from(state.get()) * config.gravity.length();
    config.physics_pipeline_active = true;
}

fn change_gravity(
    current: Res<State<Tilt>>,
    mut next: ResMut<NextState<Tilt>>,
    mut config: ResMut<RapierConfiguration>,
) {
    let direction = next_cycle(current.get()).unwrap();
    next.set(direction);
    config.gravity = Vec2::from(&direction) * config.gravity.length();
    println!("Gravity: {:?}", direction);
}

fn stabilize_on_rows(mut balls: Query<(&Transform, &Velocity, &mut ExternalForce), With<Ball>>) {
    for (tf, speed, mut ball) in balls.iter_mut() {
        let position = tf.translation.y / SIZE;
        let target = position.round();
        ball.force = Vec2::Y * (STIFFNESS * (target - position) - speed.linvel.y * DAMPING);
    }
}

fn stabilize_on_colums(mut balls: Query<(&Transform, &Velocity, &mut ExternalForce), With<Ball>>) {
    for (tf, speed, mut ball) in balls.iter_mut() {
        let position = tf.translation.x / SIZE;
        let target = position.round();
        ball.force = Vec2::X * (STIFFNESS * (target - position) - speed.linvel.x * DAMPING);
    }
}

fn track_ball_columns(
    balls: Query<&Transform, With<Ball>>,
    mut texts: Query<(&Parent, &mut Text)>,
) {
    for (tf, mut text) in texts
        .iter_mut()
        .filter_map(|(parent, text)| balls.get(parent.get()).map(|tf| (tf, text)).ok())
    {
        text.sections[0].value = format!("{:.0}", (tf.translation.y / SIZE).round() + 1.);
    }
}

fn stress_test_n(
    mut load: ResMut<TotalLoad>,
    platform: Res<Platform>,
    max_load: Res<MaxLoad>,
    balls: Query<&Transform, With<Ball>>,
    mut texts: Query<(&Index, &mut Text)>,
    mut sprites: Query<(&Index, &mut Sprite)>,
) {
    load.0 = 0;
    for (i, mut text) in texts.iter_mut().filter(|(i, _)| i.0 .1 == platform.nrows) {
        let stress = balls
            .iter()
            .map(|tf| Index::from(tf.translation))
            .filter(|index| index.0 .0 == i.0 .0)
            .map(|index| index.0 .1 + 1)
            .sum::<i32>();

        load.0 += stress;

        text.sections[0].value = stress.to_string();
        for (_, mut sprite) in sprites.iter_mut().filter(|(si, _)| *si == i) {
            sprite.color = Color::hsl(lerp(180., 0., stress as f32 / max_load.0), 0.5, 0.4);
        }
    }
}

fn stress_test_s(
    platform: Res<Platform>,
    max_load: Res<MaxLoad>,
    balls: Query<&Transform, With<Ball>>,
    mut texts: Query<(&Index, &mut Text)>,
    mut sprites: Query<(&Index, &mut Sprite)>,
) {
    for (i, mut text) in texts.iter_mut().filter(|(i, _)| i.0 .1 == -1) {
        let stress = balls
            .iter()
            .map(|tf| Index::from(tf.translation))
            .filter(|index| index.0 .0 == i.0 .0)
            .map(|index| platform.nrows - index.0 .1)
            .sum::<i32>();
        text.sections[0].value = stress.to_string();
        for (_, mut sprite) in sprites.iter_mut().filter(|(si, _)| *si == i) {
            sprite.color = Color::hsl(lerp(180., 0., stress as f32 / max_load.0), 0.5, 0.4);
        }
    }
}

fn stress_test_w(
    platform: Res<Platform>,
    max_load: Res<MaxLoad>,
    balls: Query<&Transform, With<Ball>>,
    mut texts: Query<(&Index, &mut Text)>,
    mut sprites: Query<(&Index, &mut Sprite)>,
) {
    for (i, mut text) in texts.iter_mut().filter(|(i, _)| i.0 .0 == -1) {
        let stress = balls
            .iter()
            .map(|tf| Index::from(tf.translation))
            .filter(|index| index.0 .1 == i.0 .1)
            .map(|index| platform.nrows - index.0 .0)
            .sum::<i32>();
        text.sections[0].value = stress.to_string();
        for (_, mut sprite) in sprites.iter_mut().filter(|(si, _)| *si == i) {
            sprite.color = Color::hsl(lerp(180., 0., stress as f32 / max_load.0), 0.5, 0.4);
        }
    }
}

fn stress_test_e(
    platform: Res<Platform>,
    max_load: Res<MaxLoad>,
    balls: Query<&Transform, With<Ball>>,
    mut texts: Query<(&Index, &mut Text)>,
    mut sprites: Query<(&Index, &mut Sprite)>,
) {
    for (i, mut text) in texts.iter_mut().filter(|(i, _)| i.0 .0 == platform.nrows) {
        let stress = balls
            .iter()
            .map(|tf| Index::from(tf.translation))
            .filter(|index| index.0 .1 == i.0 .1)
            .map(|index| index.0 .0 + 1)
            .sum::<i32>();
        text.sections[0].value = stress.to_string();
        for (_, mut sprite) in sprites.iter_mut().filter(|(si, _)| *si == i) {
            sprite.color = Color::hsl(lerp(180., 0., stress as f32 / max_load.0), 0.5, 0.4);
        }
    }
}

fn update_total(load: Res<TotalLoad>, mut totals: Query<&mut Text, With<Total>>) {
    totals.get_single_mut().unwrap().sections[1].value = load.0.to_string()
}

fn update(keys: Res<Input<KeyCode>>, mut exit: ResMut<Events<bevy::app::AppExit>>) {
    if keys.just_pressed(KeyCode::Q) {
        exit.send(bevy::app::AppExit);
    }
}
