use bevy::prelude::*;

use super::camera::PIXEL_SCALE;

pub struct DefaultPluginSetup;

impl Plugin for DefaultPluginSetup {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::BLACK)).add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        width: 128.0 * PIXEL_SCALE,
                        height: 128.0 * PIXEL_SCALE,
                        title: "Sigil".to_string(),
                        resizable: false,
                        ..default()
                    },
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        );
    }
}
