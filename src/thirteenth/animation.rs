use std::collections::HashSet;

use crate::{
    frequency_increaser, lerp, lerprgb, mouse, rect, toggle_running, Part, Running, Scroll, Tick,
};

use super::{Grid, Reflection};

use bevy::{prelude::*, sprite::Anchor};
use lazy_static::lazy_static;

const MOTION: f32 = 5.;
const FOUND_COLOR_TOGGLE: u8 = 2;
const SMUDGE_COLOR_TOGGLE: u8 = 2;
const FONT_SIZE: f32 = 40.;
const TILE_SIZE: f32 = 30.;
const GRID_GAP: f32 = 3. * TILE_SIZE;
const MIRROR_THICKNESS: f32 = 2.;
const MIRROR_LENGTH: f32 = 1. * TILE_SIZE;
const TOTAL_X: f32 = -2. * TILE_SIZE;
const TOTAL_Y: f32 = 0. * TILE_SIZE;
const CHECK_COLOR: Color = Color::Rgba {
    red: 0.36,
    green: 0.82,
    blue: 1.,
    alpha: 1.,
};
const FOUND_COLOR: Color = Color::GREEN;
const SMUDGE_COLOR: Color = Color::PINK;

#[derive(Debug, Resource, Default)]
struct GameState {
    part: Part,
    grids: Vec<Grid>,
    grid: usize,
    split: Reflection,
    fold: usize,
    step: Step,
    total: usize,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
enum Step {
    #[default]
    Searching,
    Smudge((u8, (usize, usize))),
    Found(u8),
    Scoring(f32),
    Done,
}

pub fn run(grids: Vec<Grid>, part: Part, frequency: f32) {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Running::default())
        .insert_resource(Tick::new(frequency))
        .insert_resource(GameState {
            part,
            grids,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update,
                mouse,
                toggle_running,
                vertical_mirror,
                horizontal_mirror,
                stripe_mover,
                cell_colorer,
                totaller,
                score_fader,
                score_mover,
                score_destroyer,
                counter,
                frequency_increaser,
            ),
        )
        .run()
}

lazy_static! {
    static ref STYLE: TextStyle = TextStyle {
        font_size: FONT_SIZE,
        color: Color::WHITE,
        ..default()
    };
}

#[derive(Debug, Component)]
struct GridComponent;

#[derive(Debug, Component)]
struct GridStripe;

#[derive(Debug, Component)]
struct Cell {
    coord: (usize, usize),
    grid: usize,
}

#[derive(Debug, Component)]
struct VerticalMirror;

#[derive(Debug, Component)]
struct HorizontalMirror;

#[derive(Debug, Component, PartialEq, Eq, Clone, Copy)]
enum VerticalMirrorHighlight {
    Left,
    Right,
}
#[derive(Debug, Component, PartialEq, Eq, Clone, Copy)]
enum HorizontalMirrorHighlight {
    Above,
    Below,
}

#[derive(Debug, Component)]
struct Total;

#[derive(Debug, Component)]
struct Score;

#[derive(Debug, Component)]
struct Counter(Reflection);

fn setup(mut cmd: Commands, state: Res<GameState>) {
    cmd.spawn((
        Scroll(0.25),
        Camera2dBundle {
            transform: Transform::from_xyz(10. * TILE_SIZE, -10. * TILE_SIZE, 0.),
            ..default()
        },
    ));

    cmd.spawn((GridStripe, SpatialBundle::default()))
        .with_children(|parent| {
            let mut last_y = 0.;
            for (g, grid) in state.grids.iter().enumerate() {
                parent
                    .spawn((
                        GridComponent,
                        SpatialBundle::from_transform(Transform::from_xyz(0., last_y, 0.)),
                    ))
                    .with_children(|parent| {
                        let rows = grid.rows();
                        for y in 0..rows {
                            for x in 0..grid.cols() {
                                parent.spawn((
                                    Cell {
                                        coord: (y, x),
                                        grid: g,
                                    },
                                    Text2dBundle {
                                        text: Text::from_section(
                                            if grid[[y, x]] == 1 { "#" } else { "." },
                                            STYLE.clone(),
                                        ),
                                        transform: Transform::from_xyz(
                                            x as f32 * TILE_SIZE + 3.,
                                            y as f32 * -TILE_SIZE - 4.,
                                            0.,
                                        ),
                                        text_anchor: Anchor::BottomLeft,
                                        ..default()
                                    },
                                ));
                            }
                        }
                    });
                last_y -= grid.rows() as f32 * TILE_SIZE;
                last_y -= GRID_GAP;
            }
        });

    let position = 2.;
    let size = state.grids[0].rows() as f32 * TILE_SIZE;
    cmd.spawn((
        VerticalMirror,
        rect(
            position * TILE_SIZE,
            size / 2.,
            2.,
            MIRROR_THICKNESS,
            size + MIRROR_LENGTH,
            Color::WHITE,
        ),
    ))
    .with_children(|parent| {
        // Fold counter
        parent.spawn((
            Counter(Reflection::Vertical),
            Text2dBundle {
                text: Text::from_section("-", STYLE.clone()),
                transform: Transform::from_xyz(0., size / 2.0 + MIRROR_LENGTH + TILE_SIZE / 2., 0.),
                text_anchor: Anchor::BottomCenter,
                ..default()
            },
        ));
        // Sides
        let w = position * TILE_SIZE;
        parent.spawn((
            VerticalMirrorHighlight::Left,
            rect(
                -w / 2.,
                0.,
                1.,
                w,
                size + MIRROR_LENGTH,
                Color::GRAY.with_a(0.5),
            ),
        ));
        parent.spawn((
            VerticalMirrorHighlight::Right,
            rect(
                w / 2.,
                0.,
                1.,
                w,
                size + MIRROR_LENGTH,
                Color::DARK_GRAY.with_a(0.5),
            ),
        ));
    });

    let size = state.grids[0].cols() as f32 * TILE_SIZE;
    cmd.spawn((
        HorizontalMirror,
        rect(
            size / 2.,
            -position * TILE_SIZE,
            2.,
            size + MIRROR_LENGTH,
            MIRROR_THICKNESS,
            Color::WHITE,
        ),
    ))
    .with_children(|parent| {
        // Fold counter
        parent.spawn((
            Counter(Reflection::Horizontal),
            Text2dBundle {
                text: Text::from_section("-", STYLE.clone()),
                transform: Transform::from_xyz(size / 2.0 + MIRROR_LENGTH + TILE_SIZE, 0., 0.),
                text_anchor: Anchor::CenterLeft,
                ..default()
            },
        ));
        // Sides
        let h = position * TILE_SIZE;
        parent.spawn((
            HorizontalMirrorHighlight::Above,
            rect(
                0.,
                -h / 2.,
                1.,
                size + MIRROR_LENGTH,
                h,
                Color::GRAY.with_a(0.5),
            ),
        ));
        parent.spawn((
            HorizontalMirrorHighlight::Below,
            rect(
                0.,
                h / 2.,
                1.,
                size + MIRROR_LENGTH,
                h,
                Color::DARK_GRAY.with_a(0.5),
            ),
        ));
    });

    cmd.spawn((
        Total,
        Text2dBundle {
            text: Text::from_sections([
                TextSection::new("Summary: ", STYLE.clone()),
                TextSection::new("---", STYLE.clone()),
            ]),
            transform: Transform::from_xyz(TOTAL_X, TOTAL_Y, 0.),
            text_anchor: Anchor::CenterRight,
            ..default()
        },
    ));
}

fn vertical_mirror(
    mut mirrors: Query<(&mut Transform, &mut Sprite, &mut Visibility), With<VerticalMirror>>,
    mut highlights: Query<
        (&VerticalMirrorHighlight, &mut Sprite, &mut Transform),
        Without<VerticalMirror>,
    >,
    state: Res<GameState>,
    time: Res<Time>,
) {
    let active = state.split == Reflection::Vertical && state.step != Step::Done;
    let fold = if active { state.fold } else { 0 };
    let cols = state.grids[state.grid].cols();
    let dt = time.delta_seconds();
    let s = state.grids[state.grid].rows() as f32 * TILE_SIZE;
    for (mut tf, mut sprite, mut visible) in mirrors.iter_mut() {
        tf.translation.x = lerp(tf.translation.x, fold as f32 * TILE_SIZE, MOTION * dt);
        tf.translation.y = -(s - TILE_SIZE - MIRROR_LENGTH) / 2.;
        *visible = if active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        if let Some(size) = sprite.custom_size.as_mut() {
            size.y = s + MIRROR_LENGTH;
        }
    }

    let target = if fold <= cols / 2 {
        fold
    } else {
        cols.saturating_sub(fold)
    } as f32
        * TILE_SIZE;
    for (side, mut sprite, mut tf) in highlights.iter_mut() {
        if let Some(size) = sprite.custom_size.as_mut() {
            size.x = lerp(size.x, target, MOTION * dt);
            size.y = s + MIRROR_LENGTH;
        }
        tf.translation.x = lerp(
            tf.translation.x,
            if *side == VerticalMirrorHighlight::Left {
                -target / 2.
            } else {
                target / 2.
            },
            MOTION * dt,
        )
    }
}

fn horizontal_mirror(
    mut mirrors: Query<(&mut Transform, &mut Sprite, &mut Visibility), With<HorizontalMirror>>,
    mut highlights: Query<
        (&HorizontalMirrorHighlight, &mut Sprite, &mut Transform),
        Without<HorizontalMirror>,
    >,
    state: Res<GameState>,
    time: Res<Time>,
) {
    let active = state.split == Reflection::Horizontal && state.step != Step::Done;
    let fold = if active { state.fold } else { 0 };
    let rows = state.grids[state.grid].rows();
    let dt = time.delta_seconds();
    let s = state.grids[state.grid].cols() as f32 * TILE_SIZE;
    for (mut tf, mut sprite, mut visible) in mirrors.iter_mut() {
        tf.translation.x = s / 2.;
        tf.translation.y = lerp(
            tf.translation.y,
            (-(fold as f32) + 1.) * TILE_SIZE,
            MOTION * dt,
        );
        *visible = if active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if let Some(size) = sprite.custom_size.as_mut() {
            size.x = s + MIRROR_LENGTH;
        }
    }
    let target = if fold <= rows / 2 {
        fold
    } else {
        rows.saturating_sub(fold)
    } as f32
        * TILE_SIZE;
    for (side, mut sprite, mut tf) in highlights.iter_mut() {
        if let Some(size) = sprite.custom_size.as_mut() {
            size.x = s + MIRROR_LENGTH;
            size.y = lerp(size.y, target, MOTION * dt);
        }
        tf.translation.y = lerp(
            tf.translation.y,
            if *side == HorizontalMirrorHighlight::Above {
                -target / 2.
            } else {
                target / 2.
            },
            MOTION * dt,
        );
    }
}

fn stripe_mover(
    time: Res<Time>,
    state: Res<GameState>,
    mut stripes: Query<&mut Transform, With<GridStripe>>,
) {
    let dt = time.delta_seconds();
    let target = state
        .grids
        .iter()
        .take(state.grid)
        .map(|grid| grid.rows() as f32 * TILE_SIZE + GRID_GAP)
        .sum::<f32>();
    for mut tf in stripes.iter_mut() {
        tf.translation.y = lerp(tf.translation.y, target, MOTION * dt);
    }
}

fn cell_colorer(time: Res<Time>, state: Res<GameState>, mut cells: Query<(&Cell, &mut Text)>) {
    let dt = time.delta_seconds();
    let grid = &state.grids[state.grid];
    let (a, b) = grid.split(state.fold, state.split);

    let n = if state.split == Reflection::Vertical {
        grid.cols()
    } else {
        grid.rows()
    };
    let offset = if state.fold <= n / 2 {
        state.fold
    } else {
        n.saturating_sub(state.fold)
    };
    let sames = (&a - &b)
        .indexed_iter()
        .filter(|(_, diff)| **diff == 0)
        .map(|((row, col), _)| match state.split {
            Reflection::Horizontal => (state.fold - 1 - row, col),
            Reflection::Vertical => (row, state.fold - col - 1),
        })
        .flat_map(|(row, col)| {
            [
                (row, col),
                match state.split {
                    Reflection::Vertical => (row, col + offset),
                    Reflection::Horizontal => (row + offset, col),
                },
            ]
            .into_iter()
        })
        .collect::<HashSet<_>>();

    for (cell, mut text) in cells.iter_mut().filter(|(cell, _)| cell.grid == state.grid) {
        let is_same = sames.contains(&cell.coord);
        let is_even = |n| n % 2 == 0;
        let opposite = match state.split {
            Reflection::Horizontal => {
                if cell.coord.0 < state.fold {
                    (
                        cell.coord.0 + 2 * (state.fold - 1 - cell.coord.0) + 1,
                        cell.coord.1,
                    )
                } else {
                    (
                        cell.coord
                            .0
                            .saturating_sub(2 * (cell.coord.0 - state.fold) + 1),
                        cell.coord.1,
                    )
                }
            }
            Reflection::Vertical => {
                if cell.coord.1 < state.fold {
                    (
                        cell.coord.0,
                        cell.coord.1 + 2 * (state.fold - 1 - cell.coord.1) + 1,
                    )
                } else {
                    (
                        cell.coord.0,
                        cell.coord
                            .1
                            .saturating_sub(2 * (cell.coord.1 - state.fold) + 1),
                    )
                }
            }
        };
        let target = match state.step {
            Step::Smudge((n, smudge))
                if (smudge == cell.coord || smudge == opposite) && is_even(n) =>
            {
                SMUDGE_COLOR
            }
            Step::Searching | Step::Smudge(_) | Step::Found(_) if is_same => CHECK_COLOR,
            Step::Found(n) if is_same && is_even(n) => FOUND_COLOR,
            Step::Searching => Color::WHITE,
            _ => Color::WHITE,
        };
        text.sections[0].style.color =
            lerprgb(text.sections[0].style.color, target, 5. * MOTION * dt);
    }
}

fn totaller(state: Res<GameState>, mut totals: Query<&mut Text, With<Total>>) {
    if state.total > 0 {
        for mut text in totals.iter_mut() {
            text.sections[1].value = format!("{:>3}", state.total);
        }
    }
}

fn score_fader(state: Res<GameState>, mut scores: Query<&mut Text, With<Score>>) {
    if let Step::Scoring(x) = state.step {
        for mut text in scores.iter_mut() {
            let color = &mut text.sections[0].style.color;
            *color = color.with_a(x);
        }
    }
}
fn score_mover(
    time: Res<Time>,
    state: Res<GameState>,
    mut scores: Query<&mut Transform, With<Score>>,
) {
    if let Step::Scoring(_) = state.step {
        let target = TOTAL_Y + 1.5 * TILE_SIZE + TILE_SIZE / 2.;
        for mut tf in scores.iter_mut() {
            tf.translation.y = lerp(tf.translation.y, target, MOTION * time.delta_seconds());
        }
    }
}

fn score_destroyer(mut cmd: Commands, state: Res<GameState>, scores: Query<Entity, With<Score>>) {
    let Step::Scoring(_) = state.step else {
        for id in scores.iter() {
            cmd.entity(id).despawn();
        }
        return;
    };
}

fn counter(state: Res<GameState>, mut counters: Query<(&mut Transform, &mut Text, &Counter)>) {
    for (mut tf, mut text, Counter(r)) in counters.iter_mut() {
        text.sections[0].value = format!("{:^2}", state.fold);
        match r {
            Reflection::Vertical => {
                tf.translation.y = state.grids[state.grid].rows() as f32 * TILE_SIZE / 2.
                    + MIRROR_LENGTH
                    - TILE_SIZE / 2.
            }
            Reflection::Horizontal => {
                tf.translation.x = state.grids[state.grid].cols() as f32 * TILE_SIZE / 2.
                    + MIRROR_LENGTH
                    + TILE_SIZE / 2.
            }
        };
    }
}

fn update(
    running: Res<Running>,
    time: Res<Time>,
    mut cmd: Commands,
    mut timer: ResMut<Tick>,
    mut state: ResMut<GameState>,
    keys: Res<Input<KeyCode>>,
    mut exit: ResMut<Events<bevy::app::AppExit>>,
) {
    if keys.just_pressed(KeyCode::Q) {
        exit.send(bevy::app::AppExit);
        return;
    }

    if !running.inner() {
        return;
    }

    if let Step::Scoring(x) = state.step {
        state.step = Step::Scoring(lerp(x, 0., MOTION * time.delta_seconds()));
    }

    if !timer.inner().tick(time.delta()).just_finished() && !keys.just_released(KeyCode::Tab) {
        return;
    }

    state.step = match (state.step, state.part) {
        (Step::Searching, Part::One) => {
            let (a, b) = state.grids[state.grid].split(state.fold, state.split);
            if !a.is_empty() && !b.is_empty() && a == b {
                Step::Found(FOUND_COLOR_TOGGLE * 2)
            } else {
                state.fold += 1;

                if state.split == Reflection::Horizontal
                    && state.fold > state.grids[state.grid].rows()
                {
                    state.split = Reflection::Vertical;
                    state.fold = 0;
                }
                Step::Searching
            }
        }
        (Step::Searching, Part::Two) => match state.grids[state.grid].find_smudge(state.split) {
            Some((index, smudge, _)) if state.fold == smudge => {
                Step::Smudge((SMUDGE_COLOR_TOGGLE * 2, index))
            }
            _ => {
                state.fold += 1;
                if state.split == Reflection::Horizontal
                    && state.fold > state.grids[state.grid].rows()
                {
                    state.split = Reflection::Vertical;
                    state.fold = 0;
                }

                Step::Searching
            }
        },
        (Step::Smudge(_), Part::One) => panic!("Smudging should only happen in Part one!"),
        (Step::Smudge((0, _)), Part::Two) => Step::Found(0),
        (Step::Smudge((n, i)), Part::Two) => Step::Smudge((n - 1, i)),
        (Step::Found(0), _) => {
            cmd.spawn((
                Score,
                Text2dBundle {
                    text: Text::from_section(
                        match state.split {
                            Reflection::Vertical => format!("+{}", state.fold),
                            Reflection::Horizontal => format!("+100*{}", state.fold),
                        },
                        TextStyle {
                            font_size: FONT_SIZE * 0.8,
                            color: Color::GRAY,
                            ..default()
                        },
                    ),
                    transform: Transform::from_xyz(TOTAL_X, TOTAL_Y + 1.5 * TILE_SIZE, 1.),
                    text_anchor: Anchor::CenterRight,
                    ..default()
                },
            ));
            state.total += match state.split {
                Reflection::Vertical => state.fold,
                Reflection::Horizontal => 100 * state.fold,
            };
            Step::Scoring(1.)
        }
        (Step::Found(x), _) => Step::Found(x - 1),
        (Step::Scoring(f), _) if f < 0.01 => {
            state.split = Reflection::default();
            state.fold = 0;
            state.grid += 1;
            if state.grid >= state.grids.len() {
                state.grid = state.grids.len() - 1;
                Step::Done
            } else {
                Step::Searching
            }
        }
        _ => state.step,
    };
}
