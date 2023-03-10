use std::f32::consts::PI;

use bevy::{input::InputSystem, prelude::*};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    game::{camera::CameraFollow, kinematic_actor::*, ldtk::EntityInstanceAdded},
    util::*,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Player>()
            .add_system(player_spawner)
            .add_system(update_level_selection)
            .add_system_to_stage(CoreStage::PreUpdate, player_input.after(InputSystem))
            .add_system(player_movement);
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

#[derive(Bundle, Default)]
pub struct PlayerBundle {
    player: Player,
    collider: Collider,
    ccd: Ccd,
    sleeping: Sleeping,
    kinematic_actor: KinematicActorBundle,
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
                    .spawn(PlayerBundle {
                        collider: Collider::cuboid(3.0, 3.0),
                        ccd: Ccd::enabled(),
                        sleeping: Sleeping::disabled(),
                        ..default()
                    })
                    .with_children(|player| {
                        player.spawn(SpriteSheetBundle {
                            transform: Transform::from_xyz(0.0, 1.0, 0.0),
                            texture_atlas: atlas_handle.clone(),
                            sprite: TextureAtlasSprite::new(0),
                            ..default()
                        });
                        player.spawn((
                            TransformBundle::from_transform(Transform::from_xyz(0.0, -32.0, 0.0)),
                            CameraFollow::instant(0),
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

fn player_input(input: Res<Input<KeyCode>>, mut query: Query<&mut KaInput, With<Player>>) {
    for mut ka_input in query.iter_mut() {
        let mut movement_input = Vec2::ZERO;
        if input.pressed(KeyCode::Right) {
            movement_input += Vec2::X
        }
        if input.pressed(KeyCode::Left) {
            movement_input += Vec2::NEG_X
        }
        ka_input.movement = movement_input;
        ka_input.jump.set(input.pressed(KeyCode::C))
    }
}

pub fn player_movement(
    mut query: Query<
        (
            Entity,
            &mut KaState,
            &mut Transform,
            &Collider,
            &KaProperties,
            &KaInput,
            &GlobalTransform,
            Option<&CollisionGroups>,
        ),
        With<Player>,
    >,
    time: Res<Time>,
    mut rapier_context: ResMut<RapierContext>,
) {
    let dt = time.delta_seconds();
    for (
        entity,
        mut state,
        mut transform,
        shape,
        props,
        input,
        global_transform,
        collision_groups,
    ) in query.iter_mut()
    {
        let movement = MovementProperties::from_props_and_state(props, &state);

        const GRAVITY_DIR: Vec2 = Vec2::NEG_Y;
        const GRAVITY_COEFFICIENT: f32 = 200.0;
        const UNITS_PER_TILE: f32 = 8.0;
        let target_velocity = input.movement * movement.speed;

        let angle_lerp = if state.velocity.length_squared() > 0.01 {
            let result = inverse_lerp(
                0.0,
                PI,
                state
                    .velocity
                    .angle_between(target_velocity - state.velocity)
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
        let velocity_change_speed = lerp(
            movement.acceleration,
            movement.friction,
            delta_interpolation,
        ) * movement.speed;

        // Apply acceleration towards wanted direction
        let current = state.velocity;
        let wanted = target_velocity.reject_from_normalized(GRAVITY_DIR);
        let grav = state.velocity.project_onto_normalized(GRAVITY_DIR);
        let mut velocity = move_towards_vec2(current, wanted + grav, velocity_change_speed * dt);
        // apply gravity
        if !state.on_ground {
            velocity += GRAVITY_DIR * GRAVITY_COEFFICIENT * dt;
        }

        if input.jump.just_pressed() && state.can_jump() {
            // Calculate required jump velocity to reach given height
            let jump_velocity =
                (props.jump_height.max(0.0) * UNITS_PER_TILE * GRAVITY_COEFFICIENT.abs() * 2.0)
                    .sqrt();
            velocity = Vec2 {
                y: jump_velocity,
                ..velocity
            };
            state.is_jumping = true;
        }

        let move_options = &MoveShapeOptions {
            up: -GRAVITY_DIR,
            autostep: Some(CharacterAutostep {
                min_width: CharacterLength::Absolute(0.5),
                max_height: CharacterLength::Absolute(0.5),
                include_dynamic_bodies: false,
            }),
            slide: false,
            max_slope_climb_angle: (50.0_f32).to_radians(),
            min_slope_slide_angle: (50.0_f32).to_radians(),
            snap_to_ground: if state.is_jumping {
                None
            } else {
                Some(CharacterLength::Absolute(1.0))
            },
            offset: CharacterLength::Absolute(0.1),
            ..MoveShapeOptions::default()
        };

        let mut move_filter = QueryFilter::new();
        let predicate = |coll_entity| coll_entity != entity;
        move_filter.predicate = Some(&predicate);

        // TODO: handle collision groups
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

        // Physics movement
        let mut remaining_velocity = velocity * dt;
        let (_scale, rotation, mut translation) = global_transform.to_scale_rotation_translation();

        const MAX_SLIDE_STEPS: u8 = 8;
        for _i in 0..MAX_SLIDE_STEPS {
            let mut colls = vec![];
            let phys_move = rapier_context.move_shape(
                remaining_velocity,
                shape,
                translation.truncate(),
                rotation.to_euler(EulerRot::ZYX).0,
                shape.raw.0.mass_properties(1.0).mass(),
                move_options,
                move_filter,
                |coll| colls.push(coll),
            );

            translation += phys_move.effective_translation.extend(0.0);
            state.on_ground = phys_move.grounded;

            // If on ground there might be some autostep/slope/snap variance so project first.
            // Otherwise, just substract effective translation from remaining velocity.
            if phys_move.grounded {
                if let Some(vel) = remaining_velocity.try_normalize() {
                    let new_remaining = remaining_velocity
                        - phys_move.effective_translation.project_onto_normalized(vel);
                    if vel.dot(new_remaining) < 0.0 {
                        remaining_velocity = Vec2::ZERO;
                    } else {
                        remaining_velocity = new_remaining;
                    }
                }
            } else {
                remaining_velocity -= phys_move.effective_translation;
            }

            for coll in colls.iter() {
                match coll.toi.status {
                    TOIStatus::Converged => {
                        remaining_velocity = remaining_velocity.reject_from(coll.toi.normal1);
                    }
                    TOIStatus::Penetrating => {
                        remaining_velocity = remaining_velocity.reject_from(coll.toi.normal1);
                        // Push slightly towards normal
                        translation += coll.toi.normal1.extend(0.0) * 0.01;
                    }
                    TOIStatus::Failed => {
                        println!("ToI failed") // DEBUG
                    }
                    TOIStatus::OutOfIterations => {
                        println!("ToI Out of Iterations") // DEBUG
                    }
                };
            }
            if remaining_velocity.abs().max_element() < 1.0e-3 {
                break;
            }
        }

        let diff = translation - global_transform.to_scale_rotation_translation().2;
        state.last_translation = diff.truncate();
        transform.translation += diff;

        // Snap to ground manually
        let ground_snap = if state.is_jumping {
            None
        } else {
            let (_scale, rotation, translation) = global_transform.to_scale_rotation_translation();
            let mut shape = shape.clone();
            shape.set_scale(Vec2::new(0.98, 1.0), 1);
            rapier_context.cast_shape(
                translation.truncate(),
                rotation.to_euler(EulerRot::ZYX).0,
                GRAVITY_DIR,
                &shape,
                1.0,
                move_filter,
            )
        };
        state.on_ground = match ground_snap {
            Some(ground_snap) => {
                if ground_snap.1.normal1.dot(-GRAVITY_DIR) > 0.0 {
                    transform.translation +=
                        GRAVITY_DIR.extend(0.0) * (ground_snap.1.toi - 0.3).max(0.0);
                    match ground_snap.1.status {
                        TOIStatus::Penetrating => {
                            transform.translation -= GRAVITY_DIR.extend(0.0) * 0.2;
                        }
                        _ => (),
                    }
                    true
                } else {
                    false
                }
            }
            None => false,
        };

        // Reset any possible jump snapping and stuff after the peak of jump
        if state.last_translation.dot(GRAVITY_DIR) >= 0.0 {
            state.is_jumping = false;
        }

        state.velocity = state.last_translation / dt;
        if state.on_ground {
            state.velocity.y = 0.0;
        }
    }
}
