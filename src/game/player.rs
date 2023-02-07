use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use super::{camera::CameraFollow, components::IdentifierBundle};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_entity::<PlayerBundle>("PLAYER");
    }
}

#[derive(Bundle, Default, LdtkEntity)]
pub struct PlayerBundle {
    camera_follow: CameraFollow,
    #[from_entity_instance]
    name: IdentifierBundle,
    worldly: Worldly,
}
