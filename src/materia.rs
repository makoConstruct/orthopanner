use bevy::prelude::*;

pub fn flat3(v: Vec2) -> Vec3 {
    Vec3::new(v.x, 0.0, v.y)
}

pub fn from_angle(v: f32) -> Vec2 {
    Vec2::new(v.cos(), v.sin())
}
pub fn rotate(v: Vec2, by: Vec2) -> Vec2 {
    Vec2::new(by.x * v.x - by.y * v.y, by.y * v.x + by.x * v.y)
}
pub fn sq(v: f32) -> f32 {
    v * v
}
