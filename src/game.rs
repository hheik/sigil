pub mod camera;
pub mod components;
pub mod debug;
pub mod default_plugin_setup;
pub mod entity_instance;
pub mod ldtk;
pub mod tile;

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use camera::*;
use debug::*;
use default_plugin_setup::*;
use entity_instance::*;
use ldtk::*;
use tile::*;

pub fn init() {
    App::new()
        .add_plugin(DefaultPluginSetup)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(DebugPlugin)
        .add_plugin(LdtkPlugin)
        .insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                load_level_neighbors: true,
            },
            set_clear_color: SetClearColor::FromLevelBackground,
            ..default()
        })
        .add_plugin(LdtkHelpers)
        .add_plugin(GameCameraPlugin)
        .add_plugin(TilePlugin)
        .add_plugin(EntityInstancePlugin)
        .insert_resource(LevelSelection::Identifier("ROOM_0".to_string()))
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: assets.load("levels/world.ldtk"),
        ..default()
    });
}
