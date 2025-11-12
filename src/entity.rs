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

impl Entity {
    pub fn new(name: &'static str,
    translation: Vector3,
    rotation: Vector3,
    scale: f32,
    motion: Motion,
    vertices: Vec<Vector3>,
    vshader: VertexShader,
    spin: Vector3,            // angular velocity (rad/s) around each local axis
    face_tangent: bool ) -> Self {
        Entity { name, translation, rotation, scale, motion, vertices, vshader, spin, face_tangent }
    }

    pub fn process_input(&mut self, window: &RaylibHandle, speed: f32, rotation_speed: f32) {
        let dt = window.get_frame_time();

        // Convención: rotation.x = pitch (alrededor de X), rotation.y = yaw (alrededor de Y), rotation.z = roll (alrededor de Z)
        let yaw = self.rotation.y;
        let pitch = self.rotation.x;

        // Vector forward en espacio mundo derivado de yaw/pitch
        // forward = (sin(yaw)*cos(pitch), -sin(pitch), cos(yaw)*cos(pitch))
        let mut forward = Vector3::new(yaw.sin() * pitch.cos(), -pitch.sin(), yaw.cos() * pitch.cos());
        if forward.length() > 0.0 { forward = forward.normalized(); }

        // Right y Up (ignoramos roll para el desplazamiento translacional)
        let world_up = Vector3::new(0.0, 1.0, 0.0);
        let mut right = forward.cross(world_up);
        if right.length() == 0.0 { right = Vector3::new(1.0, 0.0, 0.0); } else { right = right.normalized(); }
        let up = world_up; // mantener elevación en Y mundo

        // Acumular dirección de movimiento en base a la orientación del objeto
        let mut move_dir = Vector3::new(0.0, 0.0, 0.0);
        if window.is_key_down(KeyboardKey::KEY_W) { move_dir -= forward; }
        if window.is_key_down(KeyboardKey::KEY_S) { move_dir += forward; }
        if window.is_key_down(KeyboardKey::KEY_D) { move_dir -= right; }
        if window.is_key_down(KeyboardKey::KEY_A) { move_dir += right; }
        if window.is_key_down(KeyboardKey::KEY_SPACE) { move_dir += up; }
        if window.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) { move_dir -= up; }

        if move_dir.length() > 0.0 {
            move_dir = move_dir.normalized();
            self.translation += move_dir * (speed * dt);
        }

        if window.is_key_down(KeyboardKey::KEY_LEFT) { self.rotation.y -= rotation_speed; }
        if window.is_key_down(KeyboardKey::KEY_RIGHT) { self.rotation.y += rotation_speed; }
    }
}

#[derive(Clone)]
pub enum Motion {
    Static,
    Orbit { center: Vector3, radius: f32, angular_speed: f32, phase: f32 }, // world-center orbit
    OrbitAround { parent: &'static str, radius: f32, angular_speed: f32, phase: f32 }, // orbit around entity
}