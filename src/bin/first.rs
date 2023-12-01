use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    sprite::Anchor,
};

fn calibration(input: &str) -> u32 {
    input
        .lines()
        .filter_map(|line| {
            let first = line.chars().find_map(|c| c.to_digit(10))?;
            let last = line.chars().rev().find_map(|c| c.to_digit(10))?;
            Some((first, last))
        })
        .map(|(first, last)| first * 10 + last)
        .sum()
}

const FONT_SIZE: f32 = 60.0;
const CHAR_SIZE: f32 = FONT_SIZE / 2.0;
const BOX_SPEED: f32 = 4.0;
const ZOOM_SPEED: f32 = 4.0;
const ZOOM_SENSITIVITY: f32 = 0.5;
const CYCLE_TIME: f32 = 1.0;

#[derive(Default, Debug, Clone, Copy)]
enum State {
    #[default]
    Check,
    Next,
    Found(u32),
}

impl From<State> for Color {
    fn from(state: State) -> Self {
        match state {
            State::Check => Color::rgba(0.36, 0.82, 1., 0.7),
            State::Next => Color::rgba(0.93, 0.83, 0.43, 0.7),
            State::Found(_) => Color::rgba(0.54, 0.93, 0.43, 0.7),
        }
    }
}

#[derive(Debug, Component)]
struct Scroll(f32);

#[derive(Debug, Component)]
struct Line(String);
#[derive(Default, Debug, Component)]
struct Box {
    state: State,
    index: i32,
    direction: i32,
}
impl Box {
    fn step(&mut self, line: &str) {
        let c = line
            .chars()
            .skip(self.index as usize)
            .next()
            .and_then(|c| c.to_digit(10));

        self.state = match (&self.state, c) {
            (State::Check, Some(digit)) => State::Found(digit),
            (State::Check, None) => State::Next,
            (State::Next, _) => {
                self.index += self.direction;
                State::Check
            }
            (State::Found(i), _) => State::Found(*i),
        };
    }
}

impl From<&Box> for Transform {
    fn from(bx: &Box) -> Self {
        Self::from_xyz(bx.index as f32 * CHAR_SIZE, 0., 0.)
    }
}

#[derive(Resource)]
struct Tick(Timer);
#[derive(Resource)]
struct File(String);

#[derive(Default, Resource)]
struct GameState {
    run: bool,
}

fn setup(mut commands: Commands, file: Res<File>) {
    commands.spawn((
        Scroll(1.),
        Camera2dBundle {
            transform: Transform::from_xyz(200., 0., 0.),
            ..default()
        },
    ));
    let path = format!("{}/first.txt", file.0);
    let input = std::fs::read_to_string(&path).expect(&path);
    let line_scale = 1.05;
    let style = TextStyle {
        font_size: FONT_SIZE,
        color: Color::WHITE,
        ..default()
    };
    for (i, line) in input.lines().enumerate() {
        commands
            .spawn((
                Line(line.to_string()),
                Text2dBundle {
                    text: Text::from_section(line, style.clone())
                        .with_alignment(TextAlignment::Left),
                    transform: Transform::from_xyz(0., i as f32 * FONT_SIZE * line_scale, 0.),
                    text_anchor: Anchor::TopLeft,
                    ..default()
                },
            ))
            .with_children(|parent| {
                let sprite = Sprite {
                    color: State::default().into(),
                    custom_size: Some(Vec2::new(CHAR_SIZE, FONT_SIZE)),
                    anchor: Anchor::TopLeft,
                    ..default()
                };
                let first = Box {
                    index: 0,
                    direction: 1,
                    ..default()
                };
                let last = Box {
                    index: line.len() as i32 - 1,
                    direction: -1,
                    ..default()
                };
                parent.spawn((
                    SpriteBundle {
                        sprite: sprite.clone(),
                        ..default()
                    },
                    first,
                ));
                parent.spawn((
                    SpriteBundle {
                        sprite,
                        transform: (&last).into(),
                        ..default()
                    },
                    last,
                ));
            });
    }
}

fn handle_keys(mut state: ResMut<GameState>, keys: Res<Input<KeyCode>>) {
    if keys.just_released(KeyCode::Space) {
        state.run ^= true;
    }
}

fn handle_mouse(
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

fn update(
    time: Res<Time>,
    mut timer: ResMut<Tick>,
    state: Res<GameState>,
    parents: Query<&Line>,
    mut query_boxes: Query<(&Parent, &mut Box)>,
) {
    if !state.run {
        return;
    }
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }
    for (parent, mut bx) in query_boxes.iter_mut() {
        if let Ok(line) = parents.get(parent.get()) {
            bx.step(&line.0);
        }
    }
}

fn box_movement(time: Res<Time>, mut query: Query<(&Box, &mut Transform)>) {
    for (box_, mut tf) in query.iter_mut() {
        let target = Transform::from(box_);
        tf.translation.x +=
            BOX_SPEED * (target.translation.x - tf.translation.x) * time.delta_seconds();
    }
}

fn box_color(mut query: Query<(&Box, &mut Sprite)>) {
    for (b, mut sprite) in query.iter_mut() {
        sprite.color = b.state.into();
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(File(
            std::env::args()
                .skip(1)
                .next()
                .unwrap_or("sample".to_string()),
        ))
        .insert_resource(Tick(Timer::from_seconds(CYCLE_TIME, TimerMode::Repeating)))
        .insert_resource(GameState::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (update, handle_keys, handle_mouse, box_movement, box_color),
        )
        .run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample() {
        let sample = include_str!("../../sample/first.txt");
        assert_eq!(142, calibration(sample))
    }
}
