#![allow(dead_code)]

use raylib::{color::Color, math::{Vector2, Vector3}};

pub struct Fragment {
    pub position: Vector3,
    pub color: Vector3,
    pub depth: f32,
    pub obj_position: Vector3,
}

impl Fragment {
    pub fn new(x: f32, y: f32, color: Vector3, depth: f32, obj_position: Vector3) -> Self {
        Fragment {
            position: Vector3::new(x, y, depth),
            color,
            depth,
            obj_position
        }
    }
}