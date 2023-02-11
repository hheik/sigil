use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use super::{camera::CameraFollow, ldtk::EntityInstanceAdded};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Player>()
            .add_system(player_spawner)
            .add_system(update_level_selection)
            .add_system(player_mover);
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

#[derive(Bundle, Default)]
pub struct PlayerBundle {
    player: Player,
    camera_follow: CameraFollow,
    sprite_sheet: SpriteSheetBundle,
}

fn player_spawner(
    mut commands: Commands,
    mut events: EventReader<EntityInstanceAdded>,
    assets: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    for event in events.iter().filter(|e| e.instance.identifier == "PLAYER") {
        commands
            .entity(event.entity)
            .insert(Worldly::default())
            .with_children(|builder| {
                let atlas = TextureAtlas::from_grid(
                    assets.load("sprites/player.png"),
                    Vec2::splat(8.0),
                    3,
                    1,
                    Some(Vec2::splat(0.0)),
                    Some(Vec2::splat(0.0)),
                );
                let atlas_handle = atlases.add(atlas);

                builder.spawn(PlayerBundle {
                    camera_follow: CameraFollow::instant(0),
                    sprite_sheet: SpriteSheetBundle {
                        texture_atlas: atlas_handle.clone(),
                        sprite: TextureAtlasSprite::new(0),
                        ..default()
                    },
                    ..default()
                });
            });
    }
}

pub fn update_level_selection(
    level_query: Query<(&Handle<LdtkLevel>, &Transform), Without<Player>>,
    player_query: Query<&GlobalTransform, With<Player>>,
    mut level_selection: ResMut<LevelSelection>,
    ldtk_levels: Res<Assets<LdtkLevel>>,
) {
    for player_transform in &player_query {
        for (level_handle, level_transform) in &level_query {
            if let Some(ldtk_level) = ldtk_levels.get(level_handle) {
                let level_bounds = Rect {
                    min: Vec2::new(level_transform.translation.x, level_transform.translation.y),
                    max: Vec2::new(
                        level_transform.translation.x + ldtk_level.level.px_wid as f32,
                        level_transform.translation.y + ldtk_level.level.px_hei as f32,
                    ),
                };

                // TODO: Check if it's the current level
                if player_transform.translation().x < level_bounds.max.x
                    && player_transform.translation().x > level_bounds.min.x
                    && player_transform.translation().y < level_bounds.max.y
                    && player_transform.translation().y > level_bounds.min.y
                // && !level_selection.is_match(&0, &ldtk_level.level)
                {
                    *level_selection = LevelSelection::Iid(ldtk_level.level.iid.clone());
                }
            }
        }
    }
}

pub fn player_mover(
    mut query: Query<&mut Transform, With<Player>>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    for mut transform in query.iter_mut() {
        let mut movement = Vec3::ZERO;
        if input.pressed(KeyCode::Up) {
            movement += Vec3::Y
        }
        if input.pressed(KeyCode::Down) {
            movement += Vec3::NEG_Y
        }
        if input.pressed(KeyCode::Left) {
            movement += Vec3::NEG_X
        }
        if input.pressed(KeyCode::Right) {
            movement += Vec3::X
        }
        transform.translation += movement * 100.0 * time.delta_seconds();
    }
}
