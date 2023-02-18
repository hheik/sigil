use bevy::ecs::prelude::*;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_ecs_ldtk::ldtk::EnumValueDefinition;
use bevy_ecs_ldtk::{prelude::*, LdtkSystemLabel};
use bevy_rapier2d::prelude::*;
use std::collections::HashSet;

pub struct LdtkHelpers;

impl Plugin for LdtkHelpers {
    fn build(&self, app: &mut App) {
        app.add_event::<EntityInstanceAdded>()
            .register_ldtk_int_cell::<WallBundle>(1)
            .insert_resource(WordlyInstances::default())
            .insert_resource(LdtkEnum::default())
            .add_system_to_stage(CoreStage::PreUpdate, entity_instance_events)
            .add_system_to_stage(CoreStage::PostUpdate, entity_namer)
            .add_system_to_stage(
                CoreStage::PreUpdate,
                map_enum_defs.after(LdtkSystemLabel::LevelSpawning),
            )
            .add_system_to_stage(
                CoreStage::PreUpdate,
                wall_setup.after(LdtkSystemLabel::LevelSpawning),
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

#[derive(Bundle, LdtkIntCell, Default, Clone, Debug)]
pub struct WallBundle {
    wall: Wall,
}

#[derive(Component, Reflect, Default, Clone, Copy, Debug)]
#[reflect(Component)]
pub struct Wall;

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

/// Wall collider system from bevy_ecs_ldtk example.
///
/// Spawns colliders for the walls of a level
///
/// You could just insert a ColliderBundle in to the WallBundle,
/// but this spawns a different collider for EVERY wall tile.
/// This approach leads to bad performance.
///
/// Instead, by flagging the wall tiles and spawning the collisions later,
/// we can minimize the amount of colliding entities.
///
/// The algorithm used here is a nice compromise between simplicity, speed,
/// and a small number of rectangle colliders.
/// In basic terms, it will:
/// 1. consider where the walls are
/// 2. combine wall tiles into flat "plates" in each individual row
/// 3. combine the plates into rectangles across multiple rows wherever possible
/// 4. spawn colliders for each rectangle
pub fn wall_setup(
    mut commands: Commands,
    wall_query: Query<(&GridCoords, &Parent), Added<Wall>>,
    parent_query: Query<&Parent, Without<Wall>>,
    level_query: Query<(Entity, &Handle<LdtkLevel>)>,
    levels: Res<Assets<LdtkLevel>>,
) {
    /// Represents a wide wall that is 1 tile tall
    /// Used to spawn wall collisions
    #[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
    struct Plate {
        left: i32,
        right: i32,
    }

    /// A simple rectangle type representing a wall of any size
    struct Rect {
        left: i32,
        right: i32,
        top: i32,
        bottom: i32,
    }

    // Consider where the walls are
    // storing them as GridCoords in a HashSet for quick, easy lookup
    //
    // The key of this map will be the entity of the level the wall belongs to.
    // This has two consequences in the resulting collision entities:
    // 1. it forces the walls to be split along level boundaries
    // 2. it lets us easily add the collision entities as children of the appropriate level entity
    let mut level_to_wall_locations: HashMap<Entity, HashSet<GridCoords>> = HashMap::new();

    wall_query.for_each(|(&grid_coords, parent)| {
        // An intgrid tile's direct parent will be a layer entity, not the level entity
        // To get the level entity, you need the tile's grandparent.
        // This is where parent_query comes in.
        if let Ok(grandparent) = parent_query.get(parent.get()) {
            level_to_wall_locations
                .entry(grandparent.get())
                .or_default()
                .insert(grid_coords);
        }
    });

    if !wall_query.is_empty() {
        level_query.for_each(|(level_entity, level_handle)| {
            if let Some(level_walls) = level_to_wall_locations.get(&level_entity) {
                let level = levels
                    .get(level_handle)
                    .expect("Level should be loaded by this point");

                let LayerInstance {
                    c_wid: width,
                    c_hei: height,
                    grid_size,
                    ..
                } = level
                    .level
                    .layer_instances
                    .clone()
                    .expect("Level asset should have layers")[0];

                // combine wall tiles into flat "plates" in each individual row
                let mut plate_stack: Vec<Vec<Plate>> = Vec::new();

                for y in 0..height {
                    let mut row_plates: Vec<Plate> = Vec::new();
                    let mut plate_start = None;

                    // + 1 to the width so the algorithm "terminates" plates that touch the right edge
                    for x in 0..width + 1 {
                        match (plate_start, level_walls.contains(&GridCoords { x, y })) {
                            (Some(s), false) => {
                                row_plates.push(Plate {
                                    left: s,
                                    right: x - 1,
                                });
                                plate_start = None;
                            }
                            (None, true) => plate_start = Some(x),
                            _ => (),
                        }
                    }

                    plate_stack.push(row_plates);
                }

                // combine "plates" into rectangles across multiple rows
                let mut rect_builder: HashMap<Plate, Rect> = HashMap::new();
                let mut prev_row: Vec<Plate> = Vec::new();
                let mut wall_rects: Vec<Rect> = Vec::new();

                // an extra empty row so the algorithm "finishes" the rects that touch the top edge
                plate_stack.push(Vec::new());

                for (y, current_row) in plate_stack.into_iter().enumerate() {
                    for prev_plate in &prev_row {
                        if !current_row.contains(prev_plate) {
                            // remove the finished rect so that the same plate in the future starts a new rect
                            if let Some(rect) = rect_builder.remove(prev_plate) {
                                wall_rects.push(rect);
                            }
                        }
                    }
                    for plate in &current_row {
                        rect_builder
                            .entry(plate.clone())
                            .and_modify(|e| e.top += 1)
                            .or_insert(Rect {
                                bottom: y as i32,
                                top: y as i32,
                                left: plate.left,
                                right: plate.right,
                            });
                    }
                    prev_row = current_row;
                }

                commands.entity(level_entity).with_children(|level| {
                    // Spawn colliders for every rectangle..
                    // Making the collider a child of the level serves two purposes:
                    // 1. Adjusts the transforms to be relative to the level for free
                    // 2. the colliders will be despawned automatically when levels unload
                    for wall_rect in wall_rects {
                        level
                            .spawn_empty()
                            .insert(Collider::cuboid(
                                (wall_rect.right as f32 - wall_rect.left as f32 + 1.)
                                    * grid_size as f32
                                    / 2.,
                                (wall_rect.top as f32 - wall_rect.bottom as f32 + 1.)
                                    * grid_size as f32
                                    / 2.,
                            ))
                            .insert(RigidBody::Fixed)
                            .insert(Friction::new(1.0))
                            .insert(Transform::from_xyz(
                                (wall_rect.left + wall_rect.right + 1) as f32 * grid_size as f32
                                    / 2.,
                                (wall_rect.bottom + wall_rect.top + 1) as f32 * grid_size as f32
                                    / 2.,
                                0.,
                            ))
                            .insert(GlobalTransform::default());
                    }
                });
            }
        });
    }
}
