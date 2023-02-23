use crate::game::ldtk::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

pub fn pickup_setup(
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
                                RigidBody::Fixed,
                                ActiveEvents::COLLISION_EVENTS,
                                ActiveCollisionTypes::KINEMATIC_STATIC,
                                Collider::cuboid(3.5, 3.5),
                            ));
                        }
                    }
                    _ => (),
                }
            }
        });
    }
}
