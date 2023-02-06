use bevy::prelude::*;

use super::Vector2;

pub type Vector2I = Vector2<i32>;

impl Vector2I {
    pub const ZERO: Vector2I = Vector2I { x: 0, y: 0 };
    pub const ONE: Vector2I = Vector2I { x: 1, y: 1 };
    pub const UP: Vector2I = Vector2I { x: 0, y: 1 };
    pub const DOWN: Vector2I = Vector2I { x: 0, y: -1 };
    pub const LEFT: Vector2I = Vector2I { x: -1, y: 0 };
    pub const RIGHT: Vector2I = Vector2I { x: 1, y: 0 };

    pub fn angle(&self) -> f32 {
        (self.y as f32).atan2(self.x as f32)
    }
}

impl From<Vec2> for Vector2I {
    fn from(vec: Vec2) -> Self {
        Self {
            x: vec.x as i32,
            y: vec.y as i32,
        }
    }
}

impl From<Vector2I> for Vec2 {
    fn from(vec: Vector2I) -> Self {
        Vec2 {
            x: vec.x as f32,
            y: vec.y as f32,
        }
    }
}

impl From<Vector2I> for Vec3 {
    fn from(vec: Vector2I) -> Self {
        Vec3 {
            x: vec.x as f32,
            y: vec.y as f32,
            z: 0.0,
        }
    }
}
