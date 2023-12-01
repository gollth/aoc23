use aoc23::Part;
use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    sprite::Anchor,
};
use clap::Parser;

pub fn calibration(input: &str, part: Part) -> u32 {
    match part {
        Part::One => input
            .lines()
            .filter_map(|line| {
                let first = line.chars().find_map(|c| c.to_digit(10))?;
                let last = line.chars().rev().find_map(|c| c.to_digit(10))?;
                Some((first, last))
            })
            .map(|(first, last)| first * 10 + last)
            .sum(),
        Part::Two => calibration(
            &input
                .replace("one", "one1one")
                .replace("two", "two2two")
                .replace("three", "three3three")
                .replace("four", "four4four")
                .replace("five", "five5five")
                .replace("six", "six6six")
                .replace("seven", "seven7seven")
                .replace("eight", "eight8eight")
                .replace("nine", "nine9nine"),
            Part::One,
        ),
    }
}

const FONT_SIZE: f32 = 80.0;
const CHAR_SIZE: f32 = FONT_SIZE / 2.0;
const BOX_SPEED: f32 = 4.0;
const ZOOM_SPEED: f32 = 4.0;
const ZOOM_SENSITIVITY: f32 = 0.5;

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
struct Sum(Vec<Entity>);

#[derive(Debug, Component)]
struct Digit((Entity, u32));
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
            .nth(self.index as usize)
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
    let input = std::fs::read_to_string(&file.0).expect(&file.0);
    let line_scale = 1.05;
    let style = TextStyle {
        font_size: FONT_SIZE,
        color: Color::WHITE,
        ..default()
    };
    let mut digits = Vec::new();
    for (i, line) in input.lines().enumerate() {
        commands
            .spawn((
                Line(line.to_string()),
                Text2dBundle {
                    text: Text::from_section(line, style.clone())
                        .with_alignment(TextAlignment::Left),
                    transform: Transform::from_xyz(0., i as f32 * FONT_SIZE * line_scale, 0.),
                    text_anchor: Anchor::BottomLeft,
                    ..default()
                },
            ))
            .with_children(|parent| {
                let sprite = Sprite {
                    color: State::default().into(),
                    custom_size: Some(Vec2::new(CHAR_SIZE, FONT_SIZE)),
                    anchor: Anchor::BottomLeft,
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
                let left = parent
                    .spawn((
                        SpriteBundle {
                            sprite: sprite.clone(),
                            ..default()
                        },
                        first,
                    ))
                    .id();
                let right = parent
                    .spawn((
                        SpriteBundle {
                            sprite,
                            transform: (&last).into(),
                            ..default()
                        },
                        last,
                    ))
                    .id();
                let right = parent
                    .spawn((
                        Digit((right, 1)),
                        Text2dBundle {
                            text: Text::from_section(
                                "-",
                                TextStyle {
                                    font_size: FONT_SIZE,
                                    color: Color::GRAY,
                                    ..default()
                                },
                            )
                            .with_alignment(TextAlignment::Left),
                            transform: Transform::from_xyz(-CHAR_SIZE, 0., 0.),
                            text_anchor: Anchor::BottomRight,
                            ..default()
                        },
                    ))
                    .id();
                let left = parent
                    .spawn((
                        Digit((left, 10)),
                        Text2dBundle {
                            text: Text::from_section(
                                "-",
                                TextStyle {
                                    font_size: FONT_SIZE,
                                    color: Color::GRAY,
                                    ..default()
                                },
                            )
                            .with_alignment(TextAlignment::Left),
                            transform: Transform::from_xyz(-2. * CHAR_SIZE, 0., 0.),
                            text_anchor: Anchor::BottomRight,
                            ..default()
                        },
                    ))
                    .id();
                digits.push(left);
                digits.push(right);
            });
    }
    commands.spawn((
        Sum(digits),
        Text2dBundle {
            text: Text::from_section(
                "---",
                TextStyle {
                    font_size: FONT_SIZE,
                    color: Color::GRAY,
                    ..default()
                },
            )
            .with_alignment(TextAlignment::Right),
            transform: Transform::from_xyz(-CHAR_SIZE, -FONT_SIZE / 2., 0.),
            text_anchor: Anchor::TopRight,
            ..default()
        },
    ));
    commands.spawn(Text2dBundle {
        text: Text::from_section("SUM", style).with_alignment(TextAlignment::Right),
        transform: Transform::from_xyz(0., -FONT_SIZE / 2., 0.),
        text_anchor: Anchor::TopLeft,
        ..default()
    });
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

fn digit_setter(mut query: Query<(&Digit, &mut Text)>, boxes: Query<&Box>) {
    for (digit, mut text) in query.iter_mut() {
        match boxes
            .get(digit.0 .0)
            .expect("Digit to reference an Entity with a `Box` component")
            .state
        {
            State::Found(d) => {
                text.sections[0].value = format!("{d}");
                text.sections[0].style.color = Color::WHITE;
            }
            _ => {
                text.sections[0].value = "-".to_string();
                text.sections[0].style.color = Color::GRAY;
            }
        }
    }
}

fn sum_setter(mut query: Query<(&Sum, &mut Text)>, digits: Query<&Digit>, boxes: Query<&Box>) {
    for (sum, mut text) in query.iter_mut() {
        text.sections[0].style.color = Color::WHITE;
        let sum = sum
            .0
            .iter()
            .map(|id| {
                digits
                    .get(*id)
                    .expect("Sum to reference an Entity with a `Digit` component")
                    .0
            })
            .map(|digit| {
                match boxes
                    .get(digit.0)
                    .expect("Digit to reference an Entity with a `Box` component")
                    .state
                {
                    State::Found(i) => i * digit.1,
                    _ => 0,
                }
            })
            .sum::<u32>();
        if sum == 0 {
            continue;
        }
        println!("Solution A: {sum}");
        text.sections[0].value = sum.to_string();
    }
}

#[derive(Debug, Parser)]
struct Options {
    /// Path to the file with the input data
    #[clap(short, long, default_value = "sample/first.txt")]
    input: String,

    /// How often to execute each step (Hz)
    #[clap(short, long, default_value_t = 1.)]
    frequency: f32,
}

fn main() {
    let args = Options::parse();
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(File(args.input))
        .insert_resource(Tick(Timer::from_seconds(
            1. / args.frequency,
            TimerMode::Repeating,
        )))
        .insert_resource(GameState::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update,
                handle_keys,
                handle_mouse,
                box_movement,
                box_color,
                digit_setter,
                sum_setter,
            ),
        )
        .run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solution_a() {
        let sample = include_str!("../../sample/first-a.txt");
        assert_eq!(142, calibration(sample, Part::One));
    }

    #[test]
    fn solution_b() {
        let sample = include_str!("../../sample/first-b.txt");
        assert_eq!(281, calibration(sample, Part::Two));
    }
}
