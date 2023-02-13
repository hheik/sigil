use bevy::ecs::prelude::*;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_ecs_ldtk::ldtk::EnumValueDefinition;
use bevy_ecs_ldtk::*;

pub struct LdtkHelpers;

impl Plugin for LdtkHelpers {
    fn build(&self, app: &mut App) {
        app.add_event::<EntityInstanceAdded>()
            .insert_resource(WordlyInstances::default())
            .insert_resource(LdtkEnum::default())
            .add_system_to_stage(CoreStage::PreUpdate, entity_instance_events)
            .add_system_to_stage(CoreStage::PostUpdate, entity_namer)
            .add_system_to_stage(
                CoreStage::PreUpdate,
                map_enum_defs.after(LdtkSystemLabel::LevelSpawning),
            )
            .add_system_to_stage(CoreStage::PostUpdate, unique_handler);
    }
}

pub struct EntityInstanceAdded {
    pub entity: Entity,
    pub instance: EntityInstance,
}

#[derive(Resource, Default)]
pub struct WordlyInstances {
    pub def_uid_map: HashMap<i32, Entity>,
}

fn entity_instance_events(
    query: Query<(Entity, &EntityInstance), Added<EntityInstance>>,
    worldly_instances: Res<WordlyInstances>,
    mut events: EventWriter<EntityInstanceAdded>,
    mut commands: Commands,
) {
    for (entity, instance) in query.iter() {
        // Spawn the entity if it's not in the unique instances list (or if the old one is deleted)
        // TODO: Detect deleted entities safely: https://github.com/bevyengine/bevy/issues/3845
        if worldly_instances
            .def_uid_map
            .get(&instance.def_uid)
            .map_or(true, |ent| commands.get_entity(*ent).is_none())
        {
            println!("Spawned {}", instance.identifier); // DEBUG
            events.send(EntityInstanceAdded {
                entity,
                instance: instance.clone(),
            });
        }
    }
}

fn entity_namer(
    mut commands: Commands,
    mut events: EventReader<EntityInstanceAdded>,
    nameless_query: Query<(), Without<Name>>,
) {
    for event in events.iter() {
        if nameless_query.contains(event.entity) {
            commands
                .entity(event.entity)
                .insert(Name::new(event.instance.identifier.clone()));
        }
    }
}

fn unique_handler(
    query: Query<(Entity, &EntityInstance), Added<Worldly>>,
    mut worldly_instances: ResMut<WordlyInstances>,
) {
    for (entity, instance) in query.iter() {
        worldly_instances
            .def_uid_map
            .insert(instance.def_uid, entity);
    }
}

#[derive(Resource, Default)]
pub struct LdtkEnum {
    pub items: HashMap<String, EnumValueDefinition>,
    pub item_atlas: Handle<TextureAtlas>,
}

fn map_enum_defs(
    ldtk_assets: Res<Assets<LdtkAsset>>,
    mut ldtk_events: EventReader<AssetEvent<LdtkAsset>>,
    mut ldtk_enum: ResMut<LdtkEnum>,
    assets: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    for event in ldtk_events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                if let Some(ldtk_asset) = ldtk_assets.get(handle) {
                    if let Some(enum_def) = ldtk_asset
                        .project
                        .defs
                        .enums
                        .iter()
                        .find(|enum_def| enum_def.identifier == "ITEM_ID")
                    {
                        if let Some(tileset) =
                            ldtk_asset.project.defs.tilesets.iter().find(|tileset| {
                                enum_def
                                    .icon_tileset_uid
                                    .map_or(false, |uid| uid == tileset.uid)
                            })
                        {
                            if let Some(rel_path) = tileset.rel_path.clone() {
                                let atlas = TextureAtlas::from_grid(
                                    assets.load(format!("levels/{rel_path}")),
                                    Vec2::splat(tileset.tile_grid_size as f32),
                                    tileset.c_wid as usize,
                                    tileset.c_hei as usize,
                                    Some(Vec2::splat(tileset.spacing as f32)),
                                    Some(Vec2::splat(0.0)),
                                );
                                ldtk_enum.item_atlas = atlases.add(atlas);
                            }

                            for enum_value in enum_def.values.iter() {
                                ldtk_enum
                                    .items
                                    .insert(enum_value.id.clone(), enum_value.clone());
                            }
                        }
                    }
                }
            }
            _ => (),
        };
    }
}
