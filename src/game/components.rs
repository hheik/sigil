use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

#[derive(Bundle, Default)]
pub struct IdentifierBundle {
    name: Name,
}

impl From<EntityInstance> for IdentifierBundle {
    fn from(entity_instance: EntityInstance) -> Self {
        Self {
            name: Name::new(entity_instance.identifier.clone()),
        }
    }
}
