use crate::{
    mouse,
    second::{Color as C, Game},
    toggle_running, Part, Running, Scroll, Tick,
};

use bevy::{
    prelude::*,
    sprite::{Anchor, MaterialMesh2dBundle},
};
use enum_iterator::next;
use lazy_static::lazy_static;
use std::{collections::HashMap, iter::repeat, str::FromStr};

use super::BAG;

#[derive(Debug, Resource)]
struct Games(Vec<Game>);

#[derive(Debug, Default, Resource)]
struct GameState {
    bag: [usize; 3],
    game: u32,
    round: usize,
    draw: C,
    step: Step,
    checked_games: HashMap<u32, bool>,
}

#[derive(Debug, Default, Clone, Copy)]
enum Step {
    #[default]
    BagUpdate,
    ShowingResult(bool),
    Done,
}

#[derive(Debug, Default, Component, Clone, Copy, PartialEq, Eq)]
enum Draw {
    #[default]
    Unchecked,
    Checking,
    Fail,
    Success,
}

#[derive(Debug, Default, Component)]
struct Bag {
    r: Vec<Entity>,
    g: Vec<Entity>,
    b: Vec<Entity>,
}

#[derive(Debug, Component)]
struct GameId(usize);
#[derive(Debug, Component)]
struct RoundId(usize);
#[derive(Debug, Component)]
struct Label;
#[derive(Debug, Component)]
struct Sum;

#[derive(Debug, Default, Component)]
struct List;

impl From<&Draw> for Color {
    fn from(draw: &Draw) -> Self {
        match draw {
            Draw::Unchecked => Color::rgb(0.6, 0.6, 0.6),
            Draw::Checking => Color::rgb(0.36, 0.82, 1.),
            Draw::Fail => Color::RED,
            Draw::Success => Color::GREEN,
        }
    }
}

pub fn run(input: &str, frequency: f32, part: Part) {
    if part == Part::Two {
        unimplemented!("Animation for Part 2");
    }
    let games = Games(
        input
            .lines()
            .filter_map(|line| Game::from_str(line).ok())
            .collect(),
    );

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(games)
        .insert_resource(Tick::new(frequency))
        .insert_resource(Running::default())
        .insert_resource(GameState {
            game: 1,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update,
                mouse,
                draw_color,
                draw_bag,
                move_list,
                update_sum,
                toggle_running,
                highlight_draw,
                highlight_game_result,
            ),
        )
        .run()
}

const CIRCLE_RADIUS: f32 = 25.;
const FONT_SIZE: f32 = 40.;
const CHAR_SIZE: f32 = FONT_SIZE / 2.;
const PROMPT_X: f32 = -400.;
const MOVEMENT_SPEED: f32 = 5.;

lazy_static! {
    static ref STYLE: TextStyle = TextStyle {
        font_size: FONT_SIZE,
        color: Color::WHITE,
        ..default()
    };
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    games: Res<Games>,
) {
    commands.spawn((
        Scroll(0.1),
        Camera2dBundle {
            transform: Transform::from_xyz(200., 0., 0.),
            ..default()
        },
    ));

    // Right panel - Bag
    let bag_gap = 10.;
    let bag_start_y = 100.;
    let red_start_x = 250.;
    let mut bag = Bag::default();
    for y in 0..4 {
        for x in 0..3 {
            bag.r.push(
                commands
                    .spawn(MaterialMesh2dBundle {
                        mesh: meshes.add(shape::Circle::new(CIRCLE_RADIUS).into()).into(),
                        material: materials.add(ColorMaterial::from(Color::RED)),
                        transform: Transform::from_xyz(
                            red_start_x + 2.1 * x as f32 * CIRCLE_RADIUS,
                            bag_start_y - 2.1 * y as f32 * CIRCLE_RADIUS,
                            0.,
                        ),
                        ..default()
                    })
                    .id(),
            );
        }
    }

    let green_start_x = red_start_x + 3. * CIRCLE_RADIUS * 2.1 + bag_gap;
    for y in 0..5 {
        for x in 0..3 {
            if y == 4 && x != 1 {
                continue;
            }
            bag.g.push(
                commands
                    .spawn(MaterialMesh2dBundle {
                        mesh: meshes.add(shape::Circle::new(CIRCLE_RADIUS).into()).into(),
                        material: materials.add(ColorMaterial::from(Color::GREEN)),
                        transform: Transform::from_xyz(
                            green_start_x + 2.1 * x as f32 * CIRCLE_RADIUS,
                            bag_start_y - 2.1 * y as f32 * CIRCLE_RADIUS,
                            0.,
                        ),
                        ..default()
                    })
                    .id(),
            );
        }
    }

    let blue_start_x = green_start_x + 3. * CIRCLE_RADIUS * 2.1 + bag_gap;
    for y in 0..5 {
        for x in 0..3 {
            let offset = if y == 4 { CIRCLE_RADIUS } else { 0. };
            if y == 4 && x == 2 {
                continue;
            }
            bag.b.push(
                commands
                    .spawn(MaterialMesh2dBundle {
                        mesh: meshes.add(shape::Circle::new(CIRCLE_RADIUS).into()).into(),
                        material: materials.add(ColorMaterial::from(Color::BLUE)),
                        transform: Transform::from_xyz(
                            blue_start_x + 2.1 * x as f32 * CIRCLE_RADIUS + offset,
                            bag_start_y - 2.1 * y as f32 * CIRCLE_RADIUS,
                            0.,
                        ),
                        ..default()
                    })
                    .id(),
            );
        }
    }
    commands.spawn((
        bag,
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(0.7, 0.7, 0.7, 0.5),
                anchor: Anchor::TopLeft,
                custom_size: Some(Vec2::new(
                    3. * (3. * CIRCLE_RADIUS * 2.1 + bag_gap) + 2. * bag_gap,
                    2.1 * 5. * CIRCLE_RADIUS + 2. * bag_gap,
                )),
                ..default()
            },
            transform: Transform::from_xyz(
                red_start_x - CIRCLE_RADIUS - bag_gap,
                bag_start_y + CIRCLE_RADIUS + bag_gap,
                -1.,
            ),
            ..default()
        },
    ));

    // Left Panel
    commands.spawn((
        Sum,
        Text2dBundle {
            text: Text::from_section("---", STYLE.clone()).with_alignment(TextAlignment::Left),
            transform: Transform::from_xyz(PROMPT_X - CHAR_SIZE, 0., 0.),
            text_anchor: Anchor::CenterRight,
            ..default()
        },
    ));
    commands.spawn(Text2dBundle {
        text: Text::from_section(">", STYLE.clone()).with_alignment(TextAlignment::Left),
        transform: Transform::from_xyz(PROMPT_X, 0., 0.),
        ..default()
    });

    commands
        .spawn((
            List,
            TransformBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
        ))
        .with_children(|parent| {
            let mut offset = 0;
            for game in &games.0 {
                let title = format!("#{}  ", game.id);
                parent
                    .spawn((
                        GameId(game.id as usize),
                        Label,
                        Text2dBundle {
                            text: Text::from_section(&title, STYLE.clone()),
                            text_anchor: Anchor::CenterLeft,
                            transform: Transform::from_xyz(
                                PROMPT_X + FONT_SIZE,
                                -((offset) as f32) * FONT_SIZE,
                                0.,
                            ),
                            ..default()
                        },
                    ))
                    .with_children(|parent| {
                        for (ri, round) in game.rounds.iter().enumerate() {
                            let mut offset2 = 0;
                            for (color, draw) in [C::Red, C::Green, C::Blue]
                                .into_iter()
                                .filter_map(|key| round.0.get(&key).map(|value| (key, value)))
                            {
                                let label = format!("{draw} {color:?} ");
                                parent.spawn((
                                    Draw::default(),
                                    color,
                                    GameId(game.id as usize),
                                    RoundId(ri),
                                    Text2dBundle {
                                        text: Text::from_section(&label, STYLE.clone()),
                                        text_anchor: Anchor::CenterLeft,
                                        transform: Transform::from_xyz(
                                            title.len() as f32 * CHAR_SIZE
                                                + CHAR_SIZE * offset2 as f32,
                                            -(ri as f32) * FONT_SIZE,
                                            0.,
                                        ),
                                        ..default()
                                    },
                                ));
                                offset2 += label.len();
                            }
                        }
                    });
                offset += game.rounds.len() + 1;
            }
        });
}

fn draw_color(mut query: Query<(&Draw, &mut Text)>) {
    for (draw, mut text) in query.iter_mut() {
        text.sections[0].style.color = draw.into();
    }
}

fn draw_bag(
    state: Res<GameState>,
    query: Query<&Bag>,
    mut assets: ResMut<Assets<ColorMaterial>>,
    materials: Query<&Handle<ColorMaterial>>,
) {
    for bag in &query {
        for (c, n, iter) in [
            (Color::RED, state.bag[0], bag.r.iter()),
            (Color::GREEN, state.bag[1], bag.g.iter()),
            (Color::BLUE, state.bag[2], bag.b.iter()),
        ] {
            let c2 = c.with_s(0.2).with_a(0.5);
            for (color, id) in repeat(c).take(n).chain(repeat(c2)).zip(iter) {
                if let Ok(handle) = materials.get(*id) {
                    if let Some(material) = assets.get_mut(handle) {
                        material.color = color;
                    }
                }
            }
        }
    }
}

fn move_list(
    time: Res<Time>,
    state: Res<GameState>,
    games: Res<Games>,
    mut query: Query<&mut Transform, With<List>>,
) {
    let row = games
        .0
        .iter()
        .take_while(|game| game.id != state.game)
        .map(|game| game.rounds.len() + 1)
        .sum::<usize>()
        + state.round;
    for mut tf in query.iter_mut() {
        let target = (row as f32) * FONT_SIZE;
        tf.translation.y += (target - tf.translation.y) * MOVEMENT_SPEED * time.delta_seconds();
    }
}

fn highlight_draw(state: Res<GameState>, mut query: Query<(&mut Draw, &GameId, &RoundId, &C)>) {
    for (mut draw, _, _, _) in query
        .iter_mut()
        // .inspect(|c| println!(">> {c:?}"))
        .filter(|(_, gid, _, _)| state.game as usize == gid.0)
        .filter(|(_, _, rid, _)| state.round == rid.0)
        .filter(|(_, _, _, c)| state.draw == **c)
    {
        *draw = match state.step {
            Step::Done => *draw,
            Step::BagUpdate => Draw::Checking,
            Step::ShowingResult(false) => Draw::Fail,
            Step::ShowingResult(true) => Draw::Success,
        };
    }
}

fn highlight_game_result(
    state: Res<GameState>,
    mut query: Query<(&GameId, &mut Text), With<Label>>,
) {
    for (game, mut text) in query.iter_mut() {
        text.sections[0].style.color = match state.checked_games.get(&(game.0 as u32)) {
            None => Color::WHITE,
            Some(true) => Color::GREEN,
            Some(false) => Color::RED,
        };
    }
}

fn update_sum(state: Res<GameState>, mut query: Query<&mut Text, With<Sum>>) {
    for mut text in query.iter_mut() {
        text.sections[0].value = format!(
            "{sum}",
            sum = state
                .checked_games
                .iter()
                .filter(|(_, v)| **v)
                .map(|(k, _)| k)
                .sum::<u32>()
        );
    }
}

fn update(
    mut state: ResMut<GameState>,
    running: Res<Running>,
    games: Res<Games>,
    mut timer: ResMut<Tick>,
    time: Res<Time>,
) {
    if !running.inner() {
        return;
    }
    if !timer.inner().tick(time.delta()).just_finished() {
        return;
    }
    println!("State: {:?}", state);
    let game = games
        .0
        .iter()
        .find(|g| g.id == state.game)
        .unwrap_or_else(|| panic!("Game #{} to exist", state.game));
    let round = &game.rounds[state.round];
    state.step = match (state.step, round.0.get(&state.draw).as_ref()) {
        (Step::Done, _) => Step::Done,
        (Step::BagUpdate, Some(&d)) => {
            let idx = match state.draw {
                C::Red => 0,
                C::Green => 1,
                C::Blue => 2,
            };
            state.bag[idx] = *d as usize;
            Step::ShowingResult(d <= BAG.get(&state.draw).unwrap())
        }
        (Step::ShowingResult(true), _) | (Step::BagUpdate, None) => {
            let mut result = Step::BagUpdate;
            match next(&state.draw) {
                Some(n) => {
                    // Draw finished
                    state.draw = n;
                }
                None => {
                    // Round finished
                    state.draw = C::default();
                    state.round += 1;
                    state.bag = [0, 0, 0];
                    if state.round >= game.rounds.len() {
                        // Game finished
                        let gid = state.game;
                        state.checked_games.insert(gid, true);
                        state.game += 1;
                        if state.game > games.0.len() as u32 {
                            state.game = games.0.len() as u32;
                            state.round = game.rounds.len() - 1;
                            result = Step::Done;
                        } else {
                            state.round = 0;
                        }
                    }
                }
            }
            result
        }
        (Step::ShowingResult(false), _) => {
            state.draw = C::default();
            let gid = state.game;
            state.checked_games.insert(gid, false);
            state.game += 1;
            if state.game > games.0.len() as u32 {
                state.game = games.0.len() as u32;
                Step::Done
            } else {
                state.round = 0;
                state.bag = [0, 0, 0];
                Step::BagUpdate
            }
        }
    };
}
