use bevy::prelude::*;
use bevy_ecs_ldtk::LevelSelection;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_inspector_egui::quick::WorldInspectorPlugin)
            .add_plugin(bevy_rapier2d::prelude::RapierDebugRenderPlugin::default())
            .add_system(room_debug);
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
