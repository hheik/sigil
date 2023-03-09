pub mod camera;
pub mod debug;
pub mod default_plugin_setup;
pub mod entity_instance;
pub mod ldtk;
pub mod tile;

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

pub fn init() {
    App::new()
        .add_plugin(default_plugin_setup::DefaultPluginSetup)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(debug::DebugPlugin)
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

    // let size = Vec2::splat(8.0);
    // commands.spawn((
    //     Name::new("Collision test"),
    //     SpriteBundle {
    //         sprite: Sprite {
    //             custom_size: Some(size),
    //             ..default()
    //         },
    //         transform: Transform {
    //             translation: Vec3::new(-200.0, 43.0, 50.0),
    //             rotation: Quat::from_axis_angle(Vec3::Z, 45.0_f32.to_radians()),
    //             ..default()
    //         },
    //         ..default()
    //     },
    //     Collider::cuboid(size.x / 2.0, size.y / 2.0),
    // ));

    let size = Vec2::splat(40.0);
    commands.spawn((
        Name::new("Collision test"),
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(size),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(-150.0, 43.0, 50.0),
                rotation: Quat::from_axis_angle(Vec3::Z, 45.0_f32.to_radians()),
                ..default()
            },
            ..default()
        },
        Collider::cuboid(size.x / 2.0, size.y / 2.0),
    ));

    // let size = Vec2::splat(40.0);
    // commands.spawn((
    //     Name::new("Collision test"),
    //     SpriteBundle {
    //         sprite: Sprite {
    //             custom_size: Some(size),
    //             ..default()
    //         },
    //         transform: Transform {
    //             translation: Vec3::new(-150.0, 32.0, 50.0),
    //             rotation: Quat::from_axis_angle(Vec3::Z, 0.2),
    //             ..default()
    //         },
    //         ..default()
    //     },
    //     Collider::cuboid(size.x / 2.0, size.y / 2.0),
    // ));

    // let size = Vec2::splat(40.0);
    // commands.spawn((
    //     Name::new("Collision test"),
    //     SpriteBundle {
    //         sprite: Sprite {
    //             custom_size: Some(size),
    //             ..default()
    //         },
    //         transform: Transform {
    //             translation: Vec3::new(-150.0, 63.0, 50.0),
    //             rotation: Quat::from_axis_angle(Vec3::Z, -0.2),
    //             ..default()
    //         },
    //         ..default()
    //     },
    //     Collider::cuboid(size.x / 2.0, size.y / 2.0),
    // ));
}
