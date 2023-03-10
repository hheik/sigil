use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

mod input;

pub use input::*;

pub struct KinematicActorPlugin;

impl Plugin for KinematicActorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<KinematicActor>()
            .register_type::<KaInput>()
            .register_type::<KaProperties>()
            .register_type::<KaState>()
            .register_type::<KaType>();
    }
}

#[derive(Debug, SystemLabel)]
pub struct KaInputSystem;

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

#[derive(Reflect, Component, Debug)]
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
