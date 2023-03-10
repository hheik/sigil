use std::time::Duration;

use bevy::{prelude::*, time::TimeSystem};
use bevy_ecs_ldtk::LevelSelection;
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use bevy_rapier2d::prelude::*;

use super::{entity_instance::player::Player, kinematic_actor::KaState};

pub struct DebugPlugin;

#[derive(Reflect, FromReflect, Resource, Default)]
#[reflect(Resource)]
pub struct TimeDebug {
    pub fixed_timestep: Option<Duration>,
}

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TimeDebug>()
            .insert_resource(TimeDebug::default())
            .add_plugin(DebugLinesPlugin::default())
            .add_plugin(bevy_inspector_egui::quick::WorldInspectorPlugin)
            .add_plugin(bevy_rapier2d::prelude::RapierDebugRenderPlugin::default())
            .add_startup_system(init_debug)
            .add_system_to_stage(
                CoreStage::First,
                fixed_timestep.at_start().after(TimeSystem),
            )
            .add_system(player_debug)
            .add_system(room_debug);
    }
}

fn init_debug(mut time_debug: ResMut<TimeDebug>) {
    // Overrides the deltatime to be a constant value (even the unscaled value)
    // time_debug.fixed_timestep = Some(Duration::from_micros(16_666) / 1);
    time_debug.fixed_timestep = None;
}

fn player_debug(
    mut query: Query<(&KaState, &GlobalTransform), With<Player>>,
    mut debug_draw: ResMut<DebugLines>,
    time: Res<Time>,
) {
    for (kinematic_state, global_transform) in query.iter_mut() {
        let dt = time.delta_seconds();
        debug_draw.line_colored(
            global_transform.translation(),
            global_transform.translation() + kinematic_state.last_translation.extend(0.0) / dt,
            0.0,
            Color::GREEN,
        );
    }
}

fn room_debug(input: Res<Input<KeyCode>>, mut level_selection: ResMut<LevelSelection>) {
    if input.just_pressed(KeyCode::Numpad1) {
        *level_selection = LevelSelection::Index(0);
    }
    if input.just_pressed(KeyCode::Numpad2) {
        *level_selection = LevelSelection::Index(1);
    }
    if input.just_pressed(KeyCode::Numpad3) {
        *level_selection = LevelSelection::Index(2);
    }
}

fn fixed_timestep(time_debug: Res<TimeDebug>, mut time: ResMut<Time>) {
    if let Some(fixed_timestep) = time_debug.fixed_timestep {
        let instant = time.last_update().unwrap_or(time.startup()) + fixed_timestep;
        time.update_with_instant(instant);
    }
}

pub fn draw_shape(
    debug_draw: &mut ResMut<DebugLines>,
    shape: &Collider,
    translation: Vec2,
    rotation: f32,
    duration: f32,
    color: Color,
) {
    let points: Vec<Vec2> = shape_points(shape)
        .iter()
        .map(|p| Vec2::from_angle(rotation).rotate(*p) + translation)
        .collect();
    for p in points.windows(2) {
        debug_draw.line_colored(p[0].extend(0.0), p[1].extend(0.0), duration, color);
    }
}

fn shape_points(shape: &Collider) -> Vec<Vec2> {
    let mut points = vec![];
    match shape.as_typed_shape() {
        ColliderView::Cuboid(view) => {
            points.push(view.half_extents() * Vec2::new(1.0, 1.0));
            points.push(view.half_extents() * Vec2::new(1.0, -1.0));
            points.push(view.half_extents() * Vec2::new(-1.0, -1.0));
            points.push(view.half_extents() * Vec2::new(-1.0, 1.0));
        }
        _ => (),
    }
    if points.len() > 2 {
        points.push(points[0]);
    }
    points
}
