use std::f32::consts::PI;

use bevy::{
    prelude::*,
    render::mesh::VertexAttributeValues,
    sprite::{Anchor, Mesh2dHandle},
};
use itertools::Itertools;
use lazy_static::lazy_static;

use crate::{
    arc_segment, fifteenth::N, frequency_increaser, lerp, lerphsl, toggle_running, ArcSegment,
    Running, Tick,
};

use super::{parser::instructions, HashMap, Instruction, Operation};

pub fn run(frequency: f32, hashmap: HashMap, input: &str) {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Tick::new(frequency))
        .insert_resource(Running::default())
        .insert_resource(hashmap)
        .insert_resource(Instructions {
            list: instructions(input).expect("Input to be parseable").1,
            cursor: 0,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update,
                update_lens_bars,
                update_arcs,
                update_instruction_transparency,
                move_instruction_list,
                rotate_circle,
                frequency_increaser,
                toggle_running,
            ),
        )
        .run()
}

const LENS_SIZE: f32 = RADIUS / 6.;
const RADIUS: f32 = 250.;
const MOTION: f32 = 4.;
const ROTATION: f32 = 5.;
const FONT_SIZE: f32 = 40.;
const VISIBLE_INSTRUCTIONS: usize = 5;

const INSTRUCTION_LIST_OFFSET_Y: f32 = FONT_SIZE;

lazy_static! {
    static ref STYLE: TextStyle = TextStyle {
        font_size: FONT_SIZE,
        color: Color::WHITE,
        ..default()
    };
}
#[derive(Debug, Resource)]
struct Instructions {
    list: Vec<Instruction>,
    cursor: usize,
}

impl Instructions {
    fn next(&mut self) -> Option<&Instruction> {
        let x = self.list.get(self.cursor);
        self.cursor += 1;
        x
    }
}

#[derive(Debug, Component)]
struct Circle;

#[derive(Debug, Component)]
struct Lens(usize);

#[derive(Debug, Component)]
struct Bar(u8);

#[derive(Debug, Component)]
struct InstructionList;

fn color(i: usize) -> Color {
    lerphsl(
        Color::ALICE_BLUE.with_l(0.5),
        Color::ALICE_BLUE.with_l(1.),
        (i as f32 - 1.) / 9.,
    )
}

fn setup(
    mut cmd: Commands,
    instructions: Res<Instructions>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    cmd.spawn(Camera2dBundle::default());

    cmd.spawn_empty()
        .insert(SpatialBundle::default())
        .insert(Circle)
        .with_children(|parent| {
            for i in 0..N {
                let t = i as f32 / N as f32;
                parent
                    .spawn_empty()
                    .insert(SpatialBundle::default())
                    .insert(Bar(i as u8))
                    .with_children(|parent| {
                        for lens in 1..=9 {
                            parent
                                .spawn(ColorMesh2dBundle {
                                    mesh: meshes
                                        .add(arc_segment(50, &ArcSegment::default()))
                                        .into(),
                                    material: materials.add(ColorMaterial::from(color(lens))),
                                    ..default()
                                })
                                .insert(ArcSegment {
                                    phi: lerp(0., 2. * PI, t),
                                    alpha: 2. * PI / N as f32,
                                    ri: RADIUS * 0.99,
                                    ro: RADIUS,
                                })
                                .insert(Lens(lens));
                        }
                    });
            }
        });

    cmd.spawn(Text2dBundle {
        text: Text::from_section(">", STYLE.clone()),
        text_anchor: Anchor::TopRight,
        transform: Transform::from_xyz(-1.5 * FONT_SIZE, INSTRUCTION_LIST_OFFSET_Y, 0.),
        ..default()
    });
    cmd.spawn(Text2dBundle {
        text: Text::from_sections(instructions.list.iter().map(|(label, op)| {
            TextSection::new(
                format!("{label}{op}\n"),
                TextStyle {
                    color: match op {
                        Operation::Insert(_) => Color::ALICE_BLUE.with_l(0.5),
                        Operation::Remove => Color::GRAY,
                    },
                    ..STYLE.clone()
                },
            )
        })),
        transform: Transform::from_xyz(-1. * FONT_SIZE, INSTRUCTION_LIST_OFFSET_Y, 0.),
        text_anchor: Anchor::TopLeft,
        ..default()
    })
    .insert(InstructionList);
}

fn update_arcs(mut arcs: Query<(&ArcSegment, &Mesh2dHandle)>, mut assets: ResMut<Assets<Mesh>>) {
    for (arc, Mesh2dHandle(handle)) in arcs.iter_mut() {
        let mesh = assets
            .get_mut(handle.id())
            .expect("ArcSegment to have an associated mesh asset");

        let n = mesh.count_vertices() / 2;

        if let VertexAttributeValues::Float32x3(ref mut vertices) = mesh
            .attribute_mut(Mesh::ATTRIBUTE_POSITION)
            .expect("Mesh of ArcSegment to have vertex positions")
        {
            for (i, [outer_vertex, inner_vertex]) in vertices.iter_mut().array_chunks().enumerate()
            {
                let t = arc.phi + arc.alpha * (i as f32 / (n - 1) as f32);
                let (x, y) = t.sin_cos();
                *outer_vertex = [arc.ro * x, arc.ro * y, 0.];
                *inner_vertex = [arc.ri * x, arc.ri * y, 0.];
            }
        }
    }
}

fn update_lens_bars(
    time: Res<Time>,
    catalogue: Res<HashMap>,
    bars: Query<(&Bar, &Children)>,
    mut lenses: Query<(&Lens, &mut ArcSegment)>,
) {
    let dt = time.delta_seconds();
    for (Bar(label), children) in bars.iter() {
        let mut offset = RADIUS;
        for child in children {
            if let Ok((Lens(i), mut arc)) = lenses.get_mut(*child) {
                let target_size = if catalogue
                    .index(*label)
                    .map(|(_, i)| *i as usize)
                    .contains(i)
                {
                    LENS_SIZE
                } else if *i == 1 {
                    1.
                } else {
                    0.
                };
                let size = lerp(arc.ro - arc.ri, target_size, MOTION * dt);
                arc.ro = offset;
                arc.ri = offset - size;
                offset -= size;
            }
        }
    }
}

fn update_instruction_transparency(
    mut texts: Query<&mut Text, With<InstructionList>>,
    instructions: Res<Instructions>,
) {
    for (i, section) in texts
        .get_single_mut()
        .unwrap()
        .sections
        .iter_mut()
        .enumerate()
    {
        let t = 2. * (instructions.cursor as f32 - i as f32) / VISIBLE_INSTRUCTIONS as f32;
        section.style.color.set_a(1. - t.abs());
    }
}

fn move_instruction_list(
    time: Res<Time>,
    timer: Res<Tick>,
    mut texts: Query<&mut Transform, With<InstructionList>>,
    instructions: Res<Instructions>,
) {
    let mut tf = texts.get_single_mut().unwrap();
    tf.translation.y = lerp(
        tf.translation.y,
        instructions.cursor as f32 * FONT_SIZE + INSTRUCTION_LIST_OFFSET_Y,
        timer.frequency().max(MOTION) * time.delta_seconds(),
    );
}

fn rotate_circle(time: Res<Time>, mut circles: Query<&mut Transform, With<Circle>>) {
    if let Ok(mut tf) = circles.get_single_mut() {
        tf.rotate_z(ROTATION.to_radians() * time.delta_seconds());
    }
}

fn update(
    keys: Res<Input<KeyCode>>,
    running: Res<Running>,
    time: Res<Time>,
    mut timer: ResMut<Tick>,
    mut exit: ResMut<Events<bevy::app::AppExit>>,
    mut catalogue: ResMut<HashMap>,
    mut instructions: ResMut<Instructions>,
) {
    if keys.just_pressed(KeyCode::Q) {
        exit.send(bevy::app::AppExit);
    }

    let trigger = keys.just_released(KeyCode::Tab)
        || running.inner() && timer.inner().tick(time.delta()).just_finished();

    if !trigger {
        return;
    }

    if let Some(instruction) = instructions.next() {
        // println!(">> {instruction:?}");
        catalogue.process(instruction.clone());
    } else {
        println!("Processessed all instructions =)");
    }
}
