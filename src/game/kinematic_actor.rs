use std::f32::consts::PI;

use crate::util::*;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

mod input;
mod platformer;

pub use input::*;
pub use platformer::*;

pub const GRAVITY_DIR: Vec2 = Vec2::NEG_Y;
pub const GRAVITY_COEFFICIENT: f32 = 200.0;
pub const UNITS_PER_TILE: f32 = 8.0;

pub struct KinematicActorPlugin;

impl Plugin for KinematicActorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<KinematicActor>()
            .register_type::<KaInput>()
            .register_type::<KaProperties>()
            .register_type::<KaState>()
            .register_type::<KaType>()
            .add_system_to_stage(
                CoreStage::PostUpdate,
                kinematic_movement.label(KaPhysicsSystem),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                platformer_system.before(KaPhysicsSystem),
            );
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemLabel)]
pub struct KaPhysicsSystem;

#[derive(Bundle)]
pub struct KinematicActorBundle {
    pub actor: KinematicActor,
    pub input: KaInput,
    pub props: KaProperties,
    pub state: KaState,
    pub spatial: SpatialBundle,
    pub rigidbody: RigidBody,
    pub active_events: ActiveEvents,
    pub active_collisions: ActiveCollisionTypes,
}

impl Default for KinematicActorBundle {
    fn default() -> Self {
        Self {
            actor: KinematicActor::default(),
            input: KaInput::default(),
            props: KaProperties::default(),
            state: KaState::default(),
            spatial: SpatialBundle::default(),
            rigidbody: RigidBody::KinematicPositionBased,
            active_events: ActiveEvents::COLLISION_EVENTS,
            active_collisions: ActiveCollisionTypes::all(),
        }
    }
}

#[derive(Reflect, Component, Debug, Default)]
#[reflect(Component)]
pub struct KinematicActor {
    pub actor: KaType,
}

#[derive(Reflect, Component, Debug, Clone, Copy)]
#[reflect(Component)]
pub struct KaProperties {
    pub speed: f32,
    pub acceleration: f32,
    pub friction: f32,
    pub air_speed_mod: f32,
    pub air_acceleration_mod: f32,
    pub air_friction_mod: f32,
    pub jump_height: f32,
}

impl KaProperties {
    pub fn jump_velocity(&self) -> f32 {
        match velocity_required_for_jump(self.jump_height * UNITS_PER_TILE, GRAVITY_COEFFICIENT) {
            Some(velocity) => velocity,
            None => {
                warn!(
                    "Tried to calculate invalid jump velocity. jump_height: {jump_height}, UNITS_PER_TILE: {UNITS_PER_TILE}, GRAVITY_COEFFICIENT:",
                    jump_height = self.jump_height,
                );
                0.0
            }
        }
    }
}

impl Default for KaProperties {
    fn default() -> Self {
        Self {
            speed: 50.0,
            acceleration: 30.0,
            friction: 30.0,
            air_speed_mod: 1.0,
            air_acceleration_mod: 1.0,
            air_friction_mod: 1.0,
            jump_height: 2.1,
        }
    }
}

#[derive(Reflect, Component, Debug, Default, Clone, Copy)]
#[reflect(Component)]
pub struct KaState {
    pub last_translation: Vec2,
    pub velocity: Vec2,
    pub on_ground: bool,
    pub is_jumping: bool,
}

impl KaState {
    pub fn can_jump(&self) -> bool {
        self.on_ground && !self.is_jumping
    }
}

#[derive(Reflect, Default, Debug, PartialEq)]
pub enum KaType {
    #[default]
    Platformer,
}

pub struct MovementProperties {
    pub speed: f32,
    pub acceleration: f32,
    pub friction: f32,
}

impl MovementProperties {
    pub fn from_props_and_state(props: &KaProperties, state: &KaState) -> Self {
        if state.on_ground {
            Self {
                speed: props.speed,
                acceleration: props.acceleration,
                friction: props.friction,
            }
        } else {
            Self {
                speed: props.speed * props.air_speed_mod,
                acceleration: props.acceleration * props.air_acceleration_mod,
                friction: props.friction * props.air_friction_mod,
            }
        }
    }
}

pub fn kinematic_movement(
    mut query: Query<
        (
            Entity,
            &mut KaState,
            Option<&KaProperties>,
            Option<&KaInput>,
            &Collider,
            &mut Transform,
            &GlobalTransform,
            Option<&CollisionGroups>,
        ),
        With<KinematicActor>,
    >,
    time: Res<Time>,
    mut rapier_context: ResMut<RapierContext>,
) {
    let dt = time.delta_seconds();
    for (
        entity,
        mut state,
        props,
        input,
        shape,
        mut transform,
        global_transform,
        collision_groups,
    ) in query.iter_mut()
    // .filter(|q| q.0.actor == KaType::Platformer)
    {
        let props = props.cloned().unwrap_or_default();
        let input = input.cloned().unwrap_or_default();
        let movement = MovementProperties::from_props_and_state(&props, &state);
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
                velocity_required_for_jump(props.jump_height * UNITS_PER_TILE, GRAVITY_COEFFICIENT);
            if let Some(jump_velocity) = jump_velocity {
                velocity = Vec2 {
                    y: jump_velocity,
                    ..velocity
                };
                state.is_jumping = true;
            }
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
                        warn!("ToI failed: {coll:?}") // DEBUG
                    }
                    TOIStatus::OutOfIterations => {
                        warn!("ToI out of iterations: {coll:?}") // DEBUG
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
