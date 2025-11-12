use raylib::prelude::*;

use crate::VertexShader;

#[derive(Clone)]
pub struct Entity {
    pub name: &'static str,
    pub translation: Vector3,
    pub rotation: Vector3,
    pub scale: f32,
    pub motion: Motion,
    pub vertices: Vec<Vector3>,
    pub vshader: VertexShader,
    pub spin: Vector3,            // angular velocity (rad/s) around each local axis
    pub face_tangent: bool,       // if true, add tangent-facing yaw from orbital motion      // if true, add tangent-facing yaw from orbital motion
}

#[derive(Clone)]
pub enum Motion {
    Static,
    Orbit { center: Vector3, radius: f32, angular_speed: f32, phase: f32 }, // world-center orbit
    OrbitAround { parent: &'static str, radius: f32, angular_speed: f32, phase: f32 }, // orbit around entity
}