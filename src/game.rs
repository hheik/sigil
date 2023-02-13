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
    ldtk_enum: Res<LdtkEnum>,
) {
    for event in events
        .iter()
        .filter(|e| e.instance.identifier == "ITEM_PICKUP")
    {
        commands.entity(event.entity).with_children(|builder| {
            for field in event.instance.field_instances.iter() {
                match field.identifier.as_str() {
                    "ITEM_ID" => {
                        if let Some(id) = match &field.value {
                            FieldValue::Enum(value) => value,
                            _ => &None,
                        } {
                            let value_def = ldtk_enum.items.get(id).unwrap();

                            builder.spawn((
                                SpriteSheetBundle {
                                    texture_atlas: ldtk_enum.item_atlas.clone(),
                                    sprite: TextureAtlasSprite::new(
                                        value_def.tile_id.unwrap() as usize
                                    ),
                                    ..default()
                                },
                                Name::new(id.clone()),
                            ));
                        }
                    }
                    _ => (),
                }
            }
        });
    }
}
