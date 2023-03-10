use bevy::{input::InputSystem, prelude::*};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    game::{camera::CameraFollow, kinematic_actor::*, ldtk::EntityInstanceAdded},
    util::axis_from_digital,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Player>()
            .add_system(player_spawner)
            .add_system(update_level_selection)
            .add_system_to_stage(CoreStage::PreUpdate, player_input.after(InputSystem));
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

#[derive(Bundle, Default)]
pub struct PlayerBundle {
    player: Player,
    platformer: Platformer,
    collider: Collider,
    ccd: Ccd,
    sleeping: Sleeping,
    kinematic_actor: KinematicActorBundle,
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
                    assets.load("sprites/export/player.png"),
                    Vec2::splat(8.0),
                    3,
                    1,
                    Some(Vec2::splat(2.0)),
                    Some(Vec2::splat(0.0)),
                );
                let atlas_handle = atlases.add(atlas);

                builder
                    .spawn(PlayerBundle {
                        collider: Collider::cuboid(3.0, 3.0),
                        ccd: Ccd::enabled(),
                        sleeping: Sleeping::disabled(),
                        ..default()
                    })
                    .with_children(|player| {
                        player.spawn(SpriteSheetBundle {
                            transform: Transform::from_xyz(0.0, 1.0, 0.0),
                            texture_atlas: atlas_handle.clone(),
                            sprite: TextureAtlasSprite::new(0),
                            ..default()
                        });
                        player.spawn((
                            TransformBundle::from_transform(Transform::from_xyz(0.0, -32.0, 0.0)),
                            CameraFollow::instant(0),
                        ));
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

                if player_transform.translation().x < level_bounds.max.x
                    && player_transform.translation().x > level_bounds.min.x
                    && player_transform.translation().y < level_bounds.max.y
                    && player_transform.translation().y > level_bounds.min.y
                    && !level_selection.is_match(&0, &ldtk_level.level)
                {
                    *level_selection = LevelSelection::Iid(ldtk_level.level.iid.clone());
                }
            }
        }
    }
}

fn player_input(input: Res<Input<KeyCode>>, mut query: Query<&mut KaInput, With<Player>>) {
    for mut ka_input in query.iter_mut() {
        ka_input.movement = Vec2 {
            x: axis_from_digital(input.pressed(KeyCode::Left), input.pressed(KeyCode::Right)),
            y: 0.0,
        };
        ka_input.jump.set(input.pressed(KeyCode::C));
    }
}
