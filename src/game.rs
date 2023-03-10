pub mod camera;
pub mod debug;
pub mod default_plugin_setup;
pub mod entity_instance;
pub mod kinematic_actor;
pub mod ldtk;
pub mod tile;

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

pub fn init() {
    App::new()
        .add_plugin(default_plugin_setup::DefaultPluginSetup)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(debug::DebugPlugin)
        .add_plugin(LdtkPlugin)
        .insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                load_level_neighbors: true,
            },
            set_clear_color: SetClearColor::FromLevelBackground,
            ..default()
        })
        .add_plugin(ldtk::LdtkHelperPlugin)
        .add_plugin(camera::GameCameraPlugin)
        .add_plugin(tile::TilePlugin)
        .add_plugin(kinematic_actor::KinematicActorPlugin)
        .add_plugin(entity_instance::EntityInstancePlugin)
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
