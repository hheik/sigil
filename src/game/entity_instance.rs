pub mod pickup;
pub mod player;

use bevy::prelude::*;

pub struct EntityInstancePlugin;

impl Plugin for EntityInstancePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(player::PlayerPlugin)
            .add_system(pickup::pickup_setup);
    }
}
