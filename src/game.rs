pub mod camera;
pub mod components;
pub mod debug;
pub mod default_plugin_setup;
pub mod ldtk;
pub mod player;
pub mod tile;

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use camera::*;
use debug::*;
use default_plugin_setup::*;
use ldtk::*;
use player::*;
use tile::*;

pub fn init() {
    App::new()
        .add_plugin(DefaultPluginSetup)
        .add_plugin(DebugPlugin)
        .add_plugin(LdtkPlugin)
        .insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                load_level_neighbors: true,
            },
            ..default()
        })
        .add_plugin(LdtkHelpers)
        .add_plugin(GameCameraPlugin)
        .add_plugin(TilePlugin)
        .add_plugin(PlayerPlugin)
        .insert_resource(LevelSelection::Index(0))
        .add_system(pickup_setup)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: assets.load("levels/world.ldtk"),
        ..default()
    });
}

fn pickup_setup(
    mut commands: Commands,
    mut events: EventReader<EntityInstanceAdded>,
    assets: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    for event in events.iter().filter(|e| e.instance.identifier == "MONEY") {
        commands.entity(event.entity).with_children(|builder| {
            let atlas = TextureAtlas::from_grid(
                assets.load("sprites/items.png"),
                Vec2::splat(8.0),
                1,
                1,
                Some(Vec2::splat(0.0)),
                Some(Vec2::splat(0.0)),
            );
            let atlas_handle = atlases.add(atlas);

            builder.spawn(SpriteSheetBundle {
                texture_atlas: atlas_handle,
                sprite: TextureAtlasSprite::new(0),
                ..default()
            });
        });
    }
}
