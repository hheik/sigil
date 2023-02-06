use bevy::prelude::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_inspector_egui::quick::WorldInspectorPlugin);
    }
}
