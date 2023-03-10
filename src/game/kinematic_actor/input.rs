use bevy::prelude::*;

#[derive(Reflect, Default, Debug, Clone, Copy)]
pub struct KaInputButton {
    pub current: bool,
    pub previous: bool,
}

impl KaInputButton {
    /// Clears the button state, setting current and previous to deactivated.
    pub fn clear(&mut self) {
        self.current = false;
        self.previous = false;
    }

    /// Set the current state of input. Also updates just_pressed and
    /// just_released values, thus should only be called once per frame.
    /// Maybe updating the previous value should be done in a system so it won't break when called twice?
    pub fn set(&mut self, value: bool) {
        self.previous = self.current;
        self.current = value;
    }

    /// Is input currently activated?
    pub fn pressed(&self) -> bool {
        self.current
    }

    /// Was input activated this frame?
    pub fn just_pressed(&self) -> bool {
        !self.previous && self.current
    }

    /// Was input deactived this frame?
    pub fn just_released(&self) -> bool {
        self.previous && !self.current
    }
}

/// Should be applied every frame in [`CoreStage::PreUpdate`] after the [`bevy::input::InputSystem`] label
#[derive(Reflect, Component, Debug, Default, Clone, Copy)]
#[reflect(Component)]
pub struct KaInput {
    pub movement: Vec2,
    pub jump: KaInputButton,
}
