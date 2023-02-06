use crate::util::Vector2I;
use bevy::prelude::*;

pub struct TilePlugin;

pub const TILE_SIZE: f32 = 8.0;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TilePosition>()
            .add_system(position_system);
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct TilePosition(pub Vector2I);

fn position_system(mut query: Query<(&mut Transform, &TilePosition), Changed<TilePosition>>) {
    for (mut transform, position) in query.iter_mut() {
        let pos = Vec3::from(position.0) * TILE_SIZE + Vec3::new(0.0, 0.0, transform.translation.z);
        transform.translation = pos;
    }
}
