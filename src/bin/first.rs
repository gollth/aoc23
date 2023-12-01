use bevy::{prelude::*, sprite::Anchor};

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
const CYCLE_TIME: f32 = 1.0;
const COLOR_CHECK: Color = Color::Rgba {
    red: 0.36,
    green: 0.82,
    blue: 1.,
    alpha: 0.7,
};
const COLOR_NEXT: Color = Color::Rgba {
    red: 0.93,
    green: 0.83,
    blue: 0.43,
    alpha: 0.7,
};
const COLOR_FOUND: Color = Color::Rgba {
    red: 0.54,
    green: 0.93,
    blue: 0.43,
    alpha: 0.7,
};

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
            State::Check => COLOR_CHECK,
            State::Next => COLOR_NEXT,
            State::Found(_) => COLOR_FOUND,
        }
    }
}

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

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(200., 0., 0.),
        ..default()
    });
    // let input = std::fs::read_to_string("input/first.txt").expect("input/first.txt");
    let input = std::fs::read_to_string("sample/first.txt").expect("sample/first.txt");
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

fn update(
    time: Res<Time>,
    mut timer: ResMut<Tick>,
    parents: Query<&Line>,
    mut query_boxes: Query<(&Parent, &mut Box)>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }
    for (parent, mut bx) in query_boxes.iter_mut() {
        if let Ok(line) = parents.get(parent.get()) {
            bx.step(&line.0);
            println!(">> {bx:?}| {line:?}");
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
        .insert_resource(Tick(Timer::from_seconds(CYCLE_TIME, TimerMode::Repeating)))
        .add_systems(Startup, setup)
        .add_systems(Update, (update, box_movement, box_color))
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
