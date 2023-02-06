use bevy::prelude::*;

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

pub fn inverse_lerp(a: f32, b: f32, value: f32) -> f32 {
    (value - a) / (b - a)
}

pub fn vec2_lerp(a: Vec2, b: Vec2, t: f32) -> Vec2 {
    Vec2 {
        x: lerp(a.x, b.x, t),
        y: lerp(a.y, b.y, t),
    }
}

pub fn vec3_lerp(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    Vec3 {
        x: lerp(a.x, b.x, t),
        y: lerp(a.y, b.y, t),
        z: lerp(a.z, b.z, t),
    }
}

pub fn move_towards_vec2(from: Vec2, to: Vec2, amount: f32) -> Vec2 {
    let diff = to - from;
    let length = diff.length();
    if length <= f32::EPSILON {
        return from;
    }
    from + diff.normalize() * length.min(amount)
}

pub fn move_towards_vec3(from: Vec3, to: Vec3, amount: f32) -> Vec3 {
    let diff = to - from;
    let length = diff.length();
    if length <= f32::EPSILON {
        return from;
    }
    from + diff.normalize() * length.min(amount)
}
