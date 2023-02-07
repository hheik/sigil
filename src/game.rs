pub mod camera;
pub mod components;
pub mod debug;
pub mod default_plugin_setup;
pub mod player;
pub mod tile;

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use camera::*;
use debug::*;
use default_plugin_setup::*;
use player::*;
use tile::*;

pub fn init() {
    App::new()
        .add_plugin(DefaultPluginSetup)
        .add_plugin(DebugPlugin)
        .add_plugin(LdtkPlugin)
        .insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                load_level_neighbors: false,
            },
            ..default()
        })
        .add_plugin(GameCameraPlugin)
        .add_plugin(TilePlugin)
        .add_plugin(PlayerPlugin)
        .insert_resource(LevelSelection::Index(0))
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: assets.load("levels/test.ldtk"),
        ..default()
    });
}
