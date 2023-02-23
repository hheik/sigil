use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    game::{camera::CameraFollow, ldtk::EntityInstanceAdded},
    util::{inverse_lerp, lerp, move_towards_vec2},
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Player>()
            .add_system(player_spawner)
            .add_system(update_level_selection)
            .add_system(player_movement);
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

#[derive(Bundle, Default)]
pub struct PlayerBundle {
    player: Player,
    camera_follow: CameraFollow,
    sprite_sheet: SpriteSheetBundle,
    kinematic_state: KinematicState,
    props: PlayerProperties,
}

fn player_spawner(
    mut commands: Commands,
    mut events: EventReader<EntityInstanceAdded>,
    assets: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    for event in events.iter().filter(|e| e.instance.identifier == "PLAYER") {
        commands
            .entity(event.entity)
            .insert(Worldly::default())
            .with_children(|builder| {
                let atlas = TextureAtlas::from_grid(
                    assets.load("sprites/export/player.png"),
                    Vec2::splat(8.0),
                    3,
                    1,
                    Some(Vec2::splat(2.0)),
                    Some(Vec2::splat(0.0)),
                );
                let atlas_handle = atlases.add(atlas);

                builder
                    .spawn((
                        PlayerBundle {
                            camera_follow: CameraFollow::instant(0),
                            sprite_sheet: SpriteSheetBundle {
                                texture_atlas: atlas_handle.clone(),
                                sprite: TextureAtlasSprite::new(0),
                                ..default()
                            },
                            ..default()
                        },
                        RigidBody::KinematicPositionBased,
                        ActiveEvents::COLLISION_EVENTS,
                        ActiveCollisionTypes::all(),
                        Ccd::enabled(),
                        Sleeping::disabled(),
                    ))
                    .with_children(|player| {
                        player.spawn((
                            TransformBundle::from_transform(Transform::from_xyz(0.0, -1.0, 0.0)),
                            Collider::cuboid(3.0, 3.0),
                        ));
                    });
            });
    }
}

pub fn update_level_selection(
    level_query: Query<(&Handle<LdtkLevel>, &Transform), Without<Player>>,
    player_query: Query<&GlobalTransform, With<Player>>,
    mut level_selection: ResMut<LevelSelection>,
    ldtk_levels: Res<Assets<LdtkLevel>>,
) {
    for player_transform in &player_query {
        for (level_handle, level_transform) in &level_query {
            if let Some(ldtk_level) = ldtk_levels.get(level_handle) {
                let level_bounds = Rect {
                    min: Vec2::new(level_transform.translation.x, level_transform.translation.y),
                    max: Vec2::new(
                        level_transform.translation.x + ldtk_level.level.px_wid as f32,
                        level_transform.translation.y + ldtk_level.level.px_hei as f32,
                    ),
                };

                if player_transform.translation().x < level_bounds.max.x
                    && player_transform.translation().x > level_bounds.min.x
                    && player_transform.translation().y < level_bounds.max.y
                    && player_transform.translation().y > level_bounds.min.y
                    && !level_selection.is_match(&0, &ldtk_level.level)
                {
                    *level_selection = LevelSelection::Iid(ldtk_level.level.iid.clone());
                }
            }
        }
    }
}

pub fn player_movement(
    mut query: Query<
        (
            Entity,
            &mut KinematicState,
            &mut Transform,
            &PlayerProperties,
            &GlobalTransform,
            Option<&CollisionGroups>,
        ),
        With<Player>,
    >,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    shape_query: Query<&Collider, Without<Sensor>>,
    child_query: Query<&Children>,
    global_transform_query: Query<&GlobalTransform>,
    mut rapier_context: ResMut<RapierContext>,
) {
    let dt = time.delta_seconds();
    for (entity, mut kinematic_state, mut transform, props, global_transform, collision_groups) in
        query.iter_mut()
    {
        let mut movement_input = Vec2::ZERO;
        if input.pressed(KeyCode::Right) {
            movement_input += Vec2::X
        }
        if input.pressed(KeyCode::Left) {
            movement_input += Vec2::NEG_X
        }

        let (speed, acceleration, friction) = if kinematic_state
            .last_move
            .as_ref()
            .map_or(false, |last| last.grounded)
        {
            (props.speed, props.acceleration, props.friction)
        } else {
            (
                props.speed * props.air_speed_mod,
                props.acceleration * props.air_acceleration_mod,
                props.friction * props.air_friction_mod,
            )
        };

        const GRAVITY_DIR: Vec2 = Vec2::NEG_Y;
        const GRAVITY_COEFFICIENT: f32 = 2.0;

        // Calculate required jump velocity to reach given height
        let jump_velocity = (props.jump_height.max(0.0) * GRAVITY_COEFFICIENT.abs() * 2.0).sqrt();

        let current_velocity = kinematic_state
            .last_move
            .as_ref()
            .map_or(Vec2::ZERO, |last| {
                if last.grounded {
                    last.effective_translation
                        .reject_from_normalized(GRAVITY_DIR)
                } else {
                    last.effective_translation
                }
            })
            / dt;
        let target_velocity = movement_input * speed;

        let angle_lerp = if current_velocity.length_squared() > 0.01 {
            let result = inverse_lerp(
                0.0,
                PI,
                current_velocity
                    .angle_between(target_velocity - current_velocity)
                    .abs(),
            );
            if result.is_nan() {
                0.0
            } else {
                result
            }
        } else {
            0.0
        };
        let delta_interpolation = angle_lerp.clamp(0.0, 1.0);
        let velocity_change_speed = lerp(acceleration, friction, delta_interpolation) * speed;

        // apply gravity
        let mut velocity = move_towards_vec2(
            current_velocity,
            target_velocity.reject_from_normalized(GRAVITY_DIR)
                + current_velocity.project_onto_normalized(GRAVITY_DIR),
            velocity_change_speed * dt,
        ) + GRAVITY_DIR * GRAVITY_COEFFICIENT;

        if input.just_pressed(KeyCode::Space) && kinematic_state.can_jump() {
            velocity = Vec2 {
                y: jump_velocity,
                ..velocity
            };
            kinematic_state.did_jump = true;
        }

        // let (shape, shape_global_transform) = if let Ok(shape) = shape_query.get(entity) {
        let shape = if let Ok(shape) = shape_query.get(entity) {
            Some((shape, global_transform))
        } else if let Ok(children) = child_query.get(entity) {
            children.iter().find_map(|child| {
                match (
                    shape_query.get(*child).ok(),
                    global_transform_query.get(*child).ok(),
                ) {
                    (Some(s), Some(gt)) => Some((s, gt)),
                    (Some(s), None) => Some((s, global_transform)),
                    _ => None,
                }
            })
        } else {
            None
        };

        // move
        kinematic_state.last_move = if let Some((shape, shape_global_transform)) = shape {
            let (_scale, rotation, translation) =
                shape_global_transform.to_scale_rotation_translation();

            let move_options = &MoveShapeOptions {
                up: Vec2::Y,
                autostep: Some(CharacterAutostep {
                    min_width: CharacterLength::Absolute(0.5),
                    max_height: CharacterLength::Absolute(2.1),
                    include_dynamic_bodies: false,
                }),
                slide: true,
                max_slope_climb_angle: (46.0_f32).to_radians(),
                min_slope_slide_angle: (46.0_f32).to_radians(),
                snap_to_ground: if kinematic_state.did_jump {
                    None
                } else {
                    Some(CharacterLength::Absolute(2.0))
                },
                offset: CharacterLength::Absolute(0.01),
                ..MoveShapeOptions::default()
            };

            let mut filter = QueryFilter::new();
            let predicate = |coll_entity| coll_entity != entity;
            filter.predicate = Some(&predicate);

            // if let Some(collision_groups) = collision_groups {
            //     filter.groups(InteractionGroups::new(
            //         bevy_rapier2d::rapier::geometry::Group::from_bits_truncate(
            //             collision_groups.memberships.bits(),
            //         ),
            //         bevy_rapier2d::rapier::geometry::Group::from_bits_truncate(
            //             collision_groups.filters.bits(),
            //         ),
            //     ));
            // }

            let last_move: MoveShapeOutput = rapier_context.move_shape(
                velocity * dt,
                shape,
                translation.truncate(),
                rotation.to_euler(EulerRot::ZYX).0,
                shape.raw.0.mass_properties(1.0).mass(),
                move_options,
                filter,
                |_coll: CharacterCollision| (),
            );

            // Apply movement
            transform.translation += last_move.effective_translation.extend(0.0);

            Some(last_move)
        } else {
            None
        };

        // Reset any possible jump snapping and stuff after the peak of jump
        if velocity.y <= 0.0 {
            kinematic_state.did_jump = false;
        }
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct KinematicState {
    #[reflect(ignore)]
    pub last_move: Option<MoveShapeOutput>,
    pub did_jump: bool,
}

impl KinematicState {
    pub fn can_jump(&self) -> bool {
        self.last_move.as_ref().map_or(false, |last| last.grounded) && !self.did_jump
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerProperties {
    pub speed: f32,
    pub acceleration: f32,
    pub friction: f32,
    pub air_speed_mod: f32,
    pub air_acceleration_mod: f32,
    pub air_friction_mod: f32,
    pub jump_height: f32,
}

impl Default for PlayerProperties {
    fn default() -> Self {
        Self {
            speed: 75.0,
            acceleration: 20.0,
            friction: 30.0,
            air_speed_mod: 1.0,
            air_acceleration_mod: 1.0,
            air_friction_mod: 1.0,
            jump_height: 2.1,
        }
    }
}
