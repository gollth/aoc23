use super::{propagate_once, Almanac, Mapping, Resource as R};
use crate::{mouse, rect, toggle_running, Running, Scroll, Tick};

use std::{iter::once, ops::Range};

use bevy::prelude::*;
use enum_iterator::{all, next};

pub fn run(almanac: Almanac, seeds: &[Range<i128>], frequency: f32) {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::WHITE))
        .insert_resource(GameState::default())
        .insert_resource(almanac)
        .insert_resource(Seeds(seeds.to_vec()))
        .insert_resource(Tick::new(frequency))
        .insert_resource(Running::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update,
                mouse,
                toggle_running,
                range_mover,
                range_shower,
                seed_mover,
                label_mover,
            ),
        )
        .run()
}

const RANGE_COLOR: Color = Color::Rgba {
    red: 0.,
    green: 1.,
    blue: 1.,
    alpha: 1.,
};
const MOVE_SPEED: f32 = 5.;
const SHOW_SPEED: f32 = 15.;
const ROWHEIGHT: f32 = 75.;
const ROWLEN: f32 = 500.;
const FONT_SIZE: f32 = 26.;

#[derive(Default, Debug, Resource)]
struct GameState {
    res: R,
    step: Step,
    i: usize,
    selection: i32,
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
enum Step {
    #[default]
    ShowMapping,
    Propagate,
    HideMapping,
    PrepareNext,
}

#[derive(Debug, Resource)]
struct Seeds(Vec<Range<i128>>);

#[derive(Debug, Component)]
struct RangeComponent((Range<i128>, R));

#[derive(Debug, Component)]
struct Highlight;

fn setup(mut cmd: Commands, seeds: Res<Seeds>, assets: Res<AssetServer>) {
    let grey = Color::rgb(0.3, 0.3, 0.3);
    cmd.spawn((
        Scroll(0.1),
        Camera2dBundle {
            transform: Transform::from_xyz(400., 0., 0.),
            ..default()
        },
    ));
    for (y, path) in [
        "seed.png",
        "soil.png",
        "fert.png",
        "water.png",
        "light.png",
        "temperature.png",
        "humid.png",
        "location.png",
    ]
    .into_iter()
    .enumerate()
    .map(|(i, p)| (250. - i as f32 * ROWHEIGHT, p))
    {
        // Icon
        cmd.spawn(SpriteBundle {
            texture: assets.load(path),
            transform: Transform::from_xyz(-50., y, 0.),
            ..default()
        });
        // Number line
        cmd.spawn(rect(ROWLEN / 2., y, 10., ROWLEN, 2., grey));

        // Ticks
        for x in [0., 10., 20., 30., 40., 50., 60., 70., 80., 90., 100.] {
            cmd.spawn(rect(x / 100. * ROWLEN, y, 10., 2., ROWHEIGHT / 8., grey));
        }
    }

    // Seeds
    for seed in &seeds.0 {
        spawn_range(
            &mut cmd,
            seed,
            row_x(seed),
            row_y(R::Seed),
            5.,
            1.,
            R::Seed,
            RANGE_COLOR,
            (),
        );
    }

    // Label
    let seed = seeds.0.iter().min_by_key(|r| r.start).unwrap();
    cmd.spawn((
        R::Seed,
        Text2dBundle {
            text: Text::from_section(
                format!("{}", seed.start),
                TextStyle {
                    font_size: FONT_SIZE,
                    color: Color::BLACK,
                    ..default()
                },
            ),
            transform: Transform::from_xyz(row_x(seed), row_y(R::Seed) + 20., 5.),
            text_anchor: bevy::sprite::Anchor::BottomCenter,
            ..default()
        },
    ));
}

fn row_x(range: &Range<i128>) -> f32 {
    let len = (range.end - range.start) as f32;
    (range.start as f32 + len / 2.) / 100. * ROWLEN
}

#[allow(clippy::too_many_arguments)]
fn spawn_range(
    cmd: &mut Commands,
    range: &Range<i128>,
    x: f32,
    y: f32,
    z: f32,
    h: f32,
    res: R,
    color: Color,
    comps: impl Bundle,
) {
    let len = (range.end - range.start) as f32;
    let (w, h) = (len / 100. * ROWLEN, h * ROWHEIGHT / 2.);
    cmd.spawn((
        RangeComponent((range.clone(), res)),
        rect(x, y, z, w, h, color),
    ))
    .with_children(|parent| {
        parent.spawn(rect(
            0.,
            0.,
            -1.,
            w + 3.,
            h + 3.,
            Color::BLACK.with_a(color.a()),
        ));
    })
    .insert(comps);
}

fn row_y(res: R) -> f32 {
    250. - all::<R>().position(|r| r == res).unwrap() as f32 * ROWHEIGHT
}

fn range_mover(time: Res<Time>, mut query: Query<(&RangeComponent, &mut Transform)>) {
    for (c, mut tf) in query.iter_mut() {
        let (range, res) = &c.0;
        let len = (range.end - range.start) as f32;
        let (x, y) = ((range.start as f32 + len / 2.) / 100. * ROWLEN, row_y(*res));
        tf.translation.x += (x - tf.translation.x) * MOVE_SPEED * time.delta_seconds();
        tf.translation.y += (y - tf.translation.y) * MOVE_SPEED * time.delta_seconds();
    }
}

fn range_shower(
    time: Res<Time>,
    mut state: ResMut<GameState>,
    mut cmd: Commands,
    mut query: Query<(Entity, &mut Sprite), With<Highlight>>,
) {
    let mut next_step = state.step;
    for (id, mut sprite) in query.iter_mut() {
        let a = sprite.color.a();
        let ta = match state.step {
            Step::ShowMapping | Step::Propagate => 0.5,
            Step::HideMapping => 0.,
            Step::PrepareNext => a,
        };
        sprite
            .color
            .set_a(a + (ta - a) * SHOW_SPEED * time.delta_seconds());

        let target_reached = (a - ta).abs() <= 0.05;
        next_step = match state.step {
            Step::HideMapping if target_reached => {
                // Destroy highlights
                cmd.entity(id).despawn_recursive();
                Step::PrepareNext
            }
            Step::ShowMapping if target_reached => Step::Propagate,
            step => step,
        }
    }
    state.step = next_step;
}

fn seed_mover(
    keys: Res<Input<KeyCode>>,
    mut state: ResMut<GameState>,
    mut highlight: Query<&mut RangeComponent, Without<Highlight>>,
    mut sprites: Query<&mut Sprite, (Without<Highlight>, With<RangeComponent>)>,
) {
    if keys.just_released(KeyCode::Key1) {
        state.selection = 1;
        println!("Selecting Seed #1")
    }
    if keys.just_released(KeyCode::Key2) {
        state.selection = 2;
        println!("Selecting Seed #2")
    }
    if keys.just_released(KeyCode::Key3) {
        state.selection = 3;
        println!("Selecting Seed #3")
    }
    if keys.just_released(KeyCode::Key4) {
        state.selection = 4;
        println!("Selecting Seed #4")
    }
    for (i, mut sprite) in sprites.iter_mut().enumerate() {
        if i + 1 == state.selection as usize {
            sprite.color = Color::hex("#00ffff").unwrap();
        } else {
            sprite.color = RANGE_COLOR;
        }
    }
    if let Some(mut range) = highlight.iter_mut().nth((state.selection - 1) as usize) {
        if keys.just_released(KeyCode::H) {
            range.0 .0.start -= 5;
            range.0 .0.end -= 5;
        }

        if keys.just_released(KeyCode::L) {
            range.0 .0.start += 5;
            range.0 .0.end += 5;
        }
    }
}

fn label_mover(
    time: Res<Time>,
    mut texts: Query<(&mut Text, &mut Transform)>,
    ranges: Query<&RangeComponent, Without<Highlight>>,
) {
    if let Some((mut text, mut tf)) = texts.iter_mut().next() {
        if let Some((range, res)) = ranges
            .iter()
            .map(|c| c.0.clone())
            .min_by_key(|(range, _)| range.start)
        {
            let dt = time.delta_seconds();
            text.sections[0].value = format!("{}", range.start);
            tf.translation.x +=
                (row_x(&(range.start - 2..range.start + 1)) - tf.translation.x) * MOVE_SPEED * dt;
            tf.translation.y += (row_y(res) + 20. - tf.translation.y) * MOVE_SPEED * dt;
        }
    }
}

fn update(
    time: Res<Time>,
    query: Query<(Entity, &mut RangeComponent), Without<Highlight>>,
    mut cmd: Commands,
    almanac: Res<Almanac>,
    running: Res<Running>,
    mut state: ResMut<GameState>,
    mut timer: ResMut<Tick>,
) {
    if !running.inner() {
        return;
    }
    let tick = timer.inner().tick(time.delta()).just_finished();
    let nextres = next(&state.res);
    if nextres.is_none() {
        // Done
        return;
    }
    let (thisres, nextres) = (state.res, nextres.unwrap());

    let takeover = Mapping::takeover();
    let ts = almanac
        .mappings(nextres)
        .iter()
        .chain(once(&takeover))
        .collect::<Vec<_>>();
    let t = ts[state.i];
    let is_takeover = t == &takeover;
    state.step = match state.step {
        Step::ShowMapping if tick => {
            println!(
                "A) Show mapping {r:?} #{i}: {t:?}",
                r = nextres,
                i = state.i
            );
            spawn_range(
                &mut cmd,
                &t.range,
                row_x(&t.range),
                row_y(thisres),
                6.,
                1.5,
                thisres,
                Color::RED.with_a(0.),
                Highlight,
            );
            let dest = t.range.start + t.offset..t.range.end + t.offset;
            spawn_range(
                &mut cmd,
                &dest,
                row_x(&dest),
                row_y(nextres),
                6.,
                1.5,
                nextres,
                Color::LIME_GREEN.with_a(0.),
                Highlight,
            );
            Step::ShowMapping
        }
        Step::Propagate if tick => {
            let rs = query
                .iter()
                .filter(|r| r.1 .0 .1 == thisres)
                .map(|r| r.1 .0 .0.clone())
                .collect::<Vec<_>>();
            let (olds, news) = propagate_once(&rs, t);

            println!(
                "B) moving slices {r:?} #{i}: {olds:?} -> {news:?}",
                r = nextres,
                i = state.i
            );
            query
                .iter()
                .filter(|r| r.1 .0 .1 == thisres)
                .for_each(|(id, _)| cmd.entity(id).despawn_recursive());

            olds.into_iter().for_each(|r| {
                spawn_range(
                    &mut cmd,
                    &r,
                    row_x(&r),
                    row_y(thisres),
                    5.,
                    1.,
                    thisres,
                    RANGE_COLOR,
                    (),
                )
            });
            news.into_iter().for_each(|r| {
                spawn_range(
                    &mut cmd,
                    &r,
                    row_x(&(r.start - t.offset..r.end - t.offset)),
                    row_y(thisres),
                    5.,
                    1.,
                    nextres,
                    RANGE_COLOR,
                    (),
                )
            });
            if is_takeover {
                Step::PrepareNext
            } else {
                Step::HideMapping
            }
        }
        Step::PrepareNext => {
            println!("D)  prepare next {r:?} #{i}", r = nextres, i = state.i);
            state.i += 1;
            if state.i >= ts.len() {
                state.res = nextres;
                state.i = 0;
                println!("--------------------------------------");
                println!("{thisres:?} -> {nextres:?}");
                println!("--------------------------------------")
            }

            let t = ts[state.i];
            let is_takeover = t == &takeover;
            if !is_takeover {
                Step::ShowMapping
            } else {
                Step::Propagate
            }
        }
        x => x,
    };
}
