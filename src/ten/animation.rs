use crate::{mouse, toggle_running, Running, Scroll, Tick};

use super::{Coord, Maze, Pipe};

use bevy::{prelude::*, sprite::Anchor};
use std::collections::HashSet;

pub fn run(maze: Maze, frequency: f32) {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .insert_resource(maze)
        .insert_resource(GameState::default())
        .insert_resource(Running::default())
        .insert_resource(Tick::new(frequency))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update,
                mouse,
                path_counter,
                area_counter,
                toggle_running,
                pipe_colorer,
                frequency_increaser,
            ),
        )
        .run()
}

#[derive(Debug, Default, Resource)]
struct GameState {
    progress: usize,
}

impl GameState {
    fn path(&self, maze: &Maze) -> usize {
        self.progress.min(maze.path().len())
    }

    fn area(&self, maze: &Maze) -> usize {
        self.progress
            .saturating_sub(maze.path().len())
            .min(maze.inside().len())
    }
}

#[derive(Debug, Component)]
struct PathLen;

#[derive(Debug, Component)]
struct AreaLen;

const TILE: f32 = 64.;
const FONT_SIZE: f32 = 40.;

fn setup(
    mut cmd: Commands,
    assets: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    maze: Res<Maze>,
) {
    let handle = assets.load("pipes.png");
    let texture = atlases.add(TextureAtlas::from_grid(
        handle,
        Vec2::splat(TILE),
        4,
        2,
        None,
        None,
    ));
    cmd.spawn((
        Scroll(0.05),
        Camera2dBundle {
            transform: Transform::from_xyz(
                maze.start.x as f32 * TILE,
                -maze.start.y as f32 * TILE,
                0.,
            ),
            ..default()
        },
    ));

    for (coord, p) in &maze.pipes {
        cmd.spawn(pipe(coord, *p, texture.clone()));
    }

    let red_style = TextStyle {
        font_size: FONT_SIZE,
        color: Color::RED,
        ..default()
    };
    let yellow_style = TextStyle {
        font_size: FONT_SIZE,
        color: Color::YELLOW,
        ..default()
    };
    cmd.spawn((
        PathLen,
        Text2dBundle {
            text: Text::from_sections([
                TextSection::new("Path: ", red_style.clone()),
                TextSection::new("---", red_style),
            ])
            .with_alignment(TextAlignment::Right),
            transform: Transform::from_xyz(0., 0., 1.),
            text_anchor: Anchor::BottomRight,
            ..default()
        },
    ));
    cmd.spawn((
        AreaLen,
        Text2dBundle {
            text: Text::from_sections([
                TextSection::new("Area: ", yellow_style.clone()),
                TextSection::new("---", yellow_style),
            ])
            .with_alignment(TextAlignment::Right),
            transform: Transform::from_xyz(0., 0., 1.),
            text_anchor: Anchor::TopRight,
            ..default()
        },
    ));
}

fn pipe(coord: &Coord, pipe: Pipe, texture_atlas: Handle<TextureAtlas>) -> impl Bundle {
    (
        coord.clone(),
        SpriteSheetBundle {
            texture_atlas,
            sprite: TextureAtlasSprite::new(pipe.into()),
            transform: Transform::from_xyz(coord.x as f32 * TILE, -coord.y as f32 * TILE, 0.),
            ..default()
        },
    )
}

fn path_counter(state: Res<GameState>, maze: Res<Maze>, mut path: Query<&mut Text, With<PathLen>>) {
    if let Some(mut text) = path.iter_mut().next() {
        let count = state.path(&maze);
        if count > 0 {
            text.sections[1].value = format!("{}", count);
        }
    }
}
fn area_counter(state: Res<GameState>, maze: Res<Maze>, mut path: Query<&mut Text, With<AreaLen>>) {
    if let Some(mut text) = path.iter_mut().next() {
        let count = state.area(&maze);
        if count > 0 {
            text.sections[1].value = format!("{}", count);
        }
    }
}

fn update(
    running: Res<Running>,
    time: Res<Time>,
    mut timer: ResMut<Tick>,
    mut state: ResMut<GameState>,
) {
    if !running.inner() {
        return;
    }
    if !timer.inner().tick(time.delta()).just_finished() {
        return;
    }

    state.progress += 1;
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

fn pipe_colorer(
    maze: Res<Maze>,
    mut pipes: Query<(&Coord, &mut TextureAtlasSprite)>,
    state: Res<GameState>,
) {
    let path = maze
        .path()
        .iter()
        .take(state.progress)
        .collect::<HashSet<_>>();
    let inside = maze
        .inside()
        .iter()
        .take(state.progress.saturating_sub(maze.path().len()))
        .collect::<HashSet<_>>();
    for (coord, mut sprite) in pipes.iter_mut() {
        sprite.color = if path.contains(coord) {
            Color::RED
        } else if inside.contains(coord) {
            Color::YELLOW
        } else {
            Color::WHITE
        };
    }
}
