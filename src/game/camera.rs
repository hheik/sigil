use crate::util::{move_towards_vec3, vec3_lerp};
use bevy::{prelude::*, render::camera::WindowOrigin};
use bevy_ecs_ldtk::prelude::*;

pub struct GameCameraPlugin;

#[derive(StageLabel)]
pub enum CameraStages {
    /// Camera movement stage. Should run just before CoreStage::PostUpdate.
    CameraUpdate,
}

impl Plugin for GameCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_before(
            CoreStage::PostUpdate,
            CameraStages::CameraUpdate,
            SystemStage::parallel(),
        );

        app.register_type::<CameraFollow>()
            .register_type::<GameCamera>()
            .add_startup_system(camera_setup)
            .add_system_to_stage(CameraStages::CameraUpdate, camera_follow);
    }
}

#[derive(Clone, Copy, PartialEq, Reflect)]
pub enum FollowMovement {
    Instant,
    Linear(f32),
    Smooth(f32),
}

impl Default for FollowMovement {
    fn default() -> Self {
        Self::Instant
    }
}

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct GameCamera;

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct CameraFollow {
    pub priority: i32,
    pub movement: FollowMovement,
}

impl CameraFollow {
    pub fn instant(priority: i32) -> Self {
        Self {
            priority,
            movement: FollowMovement::Instant,
        }
    }

    pub fn linear(priority: i32, speed: f32) -> Self {
        Self {
            priority,
            movement: FollowMovement::Linear(speed),
        }
    }

    pub fn smooth(priority: i32, lerp: f32) -> Self {
        Self {
            priority,
            movement: FollowMovement::Smooth(lerp),
        }
    }
}

pub const PIXEL_SCALE: f32 = 6.0;

fn camera_setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Game Camera"),
        Camera2dBundle {
            projection: OrthographicProjection {
                window_origin: WindowOrigin::Center,
                scale: 1.0 / PIXEL_SCALE,
                ..default()
            },
            camera_2d: Camera2d {
                clear_color: bevy::core_pipeline::clear_color::ClearColorConfig::Custom(
                    Color::rgb(0.0, 0.0, 0.0),
                ),
            },
            ..default()
        },
        GameCamera,
    ));
}

fn camera_follow(
    mut camera_query: Query<(&mut Transform, &OrthographicProjection), With<Camera2d>>,
    follow_query: Query<(&GlobalTransform, &CameraFollow), Without<Camera2d>>,
    level_query: Query<(&GlobalTransform, &Handle<LdtkLevel>), Without<OrthographicProjection>>,
    time: Res<Time>,
    level_selection: Res<LevelSelection>,
    ldtk_levels: Res<Assets<LdtkLevel>>,
) {
    // Camera follow
    let followed = match follow_query
        .iter()
        .max_by_key(|(_transform, follow)| follow.priority)
    {
        Some(followed) => Some(followed),
        None => None,
    };

    if let Some((target, follow)) = followed {
        let target = Vec3 {
            z: 999.9,
            ..target.translation()
        };

        for (mut camera_transform, _) in camera_query.iter_mut() {
            match follow.movement {
                FollowMovement::Instant => {
                    camera_transform.translation = target;
                }
                FollowMovement::Linear(speed) => {
                    camera_transform.translation = move_towards_vec3(
                        camera_transform.translation,
                        target,
                        speed * time.delta_seconds(),
                    );
                }
                FollowMovement::Smooth(speed) => {
                    camera_transform.translation = vec3_lerp(
                        camera_transform.translation,
                        target,
                        (speed * time.delta_seconds()).min(1.0),
                    );
                }
            }
        }
    }

    // TODO: Figure out why having follow and constraint in separate ordered systems results in flickering
    // Boundaries
    for (mut camera_transform, projection) in camera_query.iter_mut() {
        for (level_transform, level_handle) in &level_query {
            if let Some(ldtk_level) = ldtk_levels.get(level_handle) {
                let level = &ldtk_level.level;
                if level_selection.is_match(&0, level) {
                    let top_limit = level_transform.translation().y + level.px_hei as f32;
                    let bottom_limit = level_transform.translation().y;
                    let left_limit = level_transform.translation().x;
                    let right_limit = level_transform.translation().x + level.px_wid as f32;

                    // vertical boundaries
                    camera_transform.translation.y += (bottom_limit
                        - (projection.bottom * projection.scale + camera_transform.translation.y))
                        .max(0.0);
                    camera_transform.translation.y += (top_limit
                        - (projection.top * projection.scale + camera_transform.translation.y))
                        .min(0.0);

                    // horizontal boundaries
                    camera_transform.translation.x += (left_limit
                        - (projection.left * projection.scale + camera_transform.translation.x))
                        .max(0.0);
                    camera_transform.translation.x += (right_limit
                        - (projection.right * projection.scale + camera_transform.translation.x))
                        .min(0.0);
                }
            }
        }
    }
}
