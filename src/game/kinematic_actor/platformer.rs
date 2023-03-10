use bevy::prelude::*;
use bevy_rapier2d::na::clamp;

use super::*;

#[derive(Reflect, Component, Default, Debug)]
#[reflect(Component)]
pub struct Platformer {
    is_short_hopping: bool,
}

// Short hop gravity is added after normal gravity,
// so treat a multiplier of 0 as normal gravity and 1 as double

/// Gravity multiplier added at the start of jump (linearly interpolated)
const SHORT_HOP_GRAVITY_MULT_START: f32 = 3.0;
/// Gravity multiplier added at the end of jump (linearly interpolated)
const SHORT_HOP_GRAVITY_MULT_STOP: f32 = 0.5;

pub fn platformer_system(
    mut query: Query<(&mut KaState, &KaInput, &KaProperties, &mut Platformer)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for (mut state, input, props, mut platformer) in query.iter_mut() {
        if !platformer.is_short_hopping && state.is_jumping && input.jump.just_released() {
            platformer.is_short_hopping = true;
        }

        if platformer.is_short_hopping && !state.is_jumping {
            platformer.is_short_hopping = false;
        }

        if platformer.is_short_hopping {
            let gravity_mult = lerp(
                SHORT_HOP_GRAVITY_MULT_STOP,
                SHORT_HOP_GRAVITY_MULT_START,
                clamp(
                    state.velocity.project_onto(GRAVITY_DIR).length() / props.jump_velocity(),
                    0.0,
                    1.0,
                ),
            );
            state.velocity += GRAVITY_DIR * GRAVITY_COEFFICIENT * gravity_mult * dt;
        }
    }
}
