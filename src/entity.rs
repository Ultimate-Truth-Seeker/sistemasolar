use raylib::prelude::*;

use crate::{VertexShader, shaders::FragmentShader};

#[derive(Clone)]
pub struct Entity {
    pub name: &'static str,
    pub translation: Vector3,
    pub rotation: Vector3,
    pub scale: f32,

    // Base local de la entidad (orientación en mundo)
    pub forward: Vector3,
    pub right: Vector3,
    pub up: Vector3,

    pub motion: Motion,
    pub vertices: Vec<Vector3>,
    pub vshader: VertexShader,
    pub fshader: FragmentShader,
    pub spin: Vector3,            // angular velocity (rad/s) around each local axis
    pub face_tangent: bool,       // if true, add tangent-facing yaw from orbital motion      // if true, add tangent-facing yaw from orbital motion
}

impl Entity {
    fn rotate_around_axis(v: Vector3, axis: Vector3, angle: f32) -> Vector3 {
        let len = axis.length();
        if len == 0.0 {
            return v;
        }
        let a = axis / len;
        let cos_t = angle.cos();
        let sin_t = angle.sin();
        // a × v
        let cross = a.cross(v);
        // a · v
        let dot = a.dot(v);
        v * cos_t + cross * sin_t + a * (dot * (1.0 - cos_t))
    }

    pub fn new(
        name: &'static str,
        translation: Vector3,
        rotation: Vector3,
        scale: f32,
        motion: Motion,
        vertices: Vec<Vector3>,
        vshader: VertexShader,
        fshader: FragmentShader,
        spin: Vector3,            // angular velocity (rad/s) around each local axis
        face_tangent: bool,
    ) -> Self {
        // Inicializar base local a partir de los ángulos Euler iniciales
        let pitch = rotation.x;
        let yaw   = rotation.y;
        let roll  = rotation.z;

        // Forward a partir de yaw/pitch
        let mut forward = Vector3::new(
            yaw.sin() * pitch.cos(),
            -pitch.sin(),
            yaw.cos() * pitch.cos(),
        );
        if forward.length() > 0.0 {
            forward = forward.normalized();
        }

        // Base sin roll usando Y-up mundial
        let world_up = Vector3::new(0.0, 1.0, 0.0);
        let mut right0 = world_up.cross(forward);
        if right0.length() == 0.0 {
            right0 = Vector3::new(1.0, 0.0, 0.0);
        } else {
            right0 = right0.normalized();
        }
        let mut up0 = forward.cross(right0);
        if up0.length() == 0.0 {
            up0 = world_up;
        } else {
            up0 = up0.normalized();
        }

        // Aplicar roll inicial sobre (right0, up0)
        let cr = roll.cos();
        let sr = roll.sin();
        let right = right0 * cr + up0 * sr;
        let up    = -right0 * sr + up0 * cr;

        Entity {
            name,
            translation,
            rotation,
            scale,
            forward,
            right,
            up,
            motion,
            vertices,
            vshader,
            fshader,
            spin,
            face_tangent,
        }
    }

    pub fn process_input(&mut self, window: &RaylibHandle, speed: f32, rotation_speed: f32) -> (Vector3, Vector3) {
        let dt = window.get_frame_time();

        // 1) Base local actual (verdad de la orientación)
        let mut f = if self.forward.length() > 0.0 {
            self.forward
        } else {
            Vector3::new(0.0, 0.0, 1.0)
        };
        let mut r = if self.right.length() > 0.0 {
            self.right
        } else {
            Vector3::new(1.0, 0.0, 0.0)
        };
        let mut u = if self.up.length() > 0.0 {
            self.up
        } else {
            Vector3::new(0.0, 1.0, 0.0)
        };

        // 2) Aplicar giros en la base local según la entrada
        let dangle = rotation_speed * dt;

        // Yaw: rotar alrededor del eje local 'up' (u)
        if window.is_key_down(KeyboardKey::KEY_LEFT) {
            f = Self::rotate_around_axis(f, u,  dangle);
            r = Self::rotate_around_axis(r, u,  dangle);
        }
        if window.is_key_down(KeyboardKey::KEY_RIGHT) {
            f = Self::rotate_around_axis(f, u, -dangle);
            r = Self::rotate_around_axis(r, u, -dangle);
        }

        // Pitch: rotar alrededor del eje local 'right' (r)
        if window.is_key_down(KeyboardKey::KEY_UP) {
            f = Self::rotate_around_axis(f, r,  dangle);
            u = Self::rotate_around_axis(u, r,  dangle);
        }
        if window.is_key_down(KeyboardKey::KEY_DOWN) {
            f = Self::rotate_around_axis(f, r, -dangle);
            u = Self::rotate_around_axis(u, r, -dangle);
        }

        // Roll: rotar alrededor del eje local 'forward' (f)
        if window.is_key_down(KeyboardKey::KEY_Q) {
            r = Self::rotate_around_axis(r, f,  dangle);
            u = Self::rotate_around_axis(u, f,  dangle);
        }
        if window.is_key_down(KeyboardKey::KEY_E) {
            r = Self::rotate_around_axis(r, f, -dangle);
            u = Self::rotate_around_axis(u, f, -dangle);
        }

        // 3) Re-ortonormalizar base para evitar deriva numérica (Gram–Schmidt local)
        // Mantener f como eje principal
        f = f.normalized();

        // Hacer r perpendicular a f, pero lo más cercano posible al r previo
        let mut r_proj = r - f * f.dot(r);
        if r_proj.length() < 1e-6 {
            // Si por alguna razón r se volvió casi paralelo a f, elegir un eje auxiliar suave
            let aux = if f.y.abs() < 0.99 {
                Vector3::new(0.0, 1.0, 0.0)
            } else {
                Vector3::new(1.0, 0.0, 0.0)
            };
            r_proj = (aux - f * f.dot(aux)).normalized();
        } else {
            r_proj = r_proj.normalized();
        }

        // u = f × r, automáticamente ortogonal a ambos
        let mut u_proj = f.cross(r_proj);
        if u_proj.length() < 1e-6 {
            // Fallback muy raro: regenerar usando otro auxiliar
            let aux = Vector3::new(0.0, 1.0, 0.0);
            r_proj = (aux - f * f.dot(aux)).normalized();
            u_proj = f.cross(r_proj).normalized();
        } else {
            u_proj = u_proj.normalized();
        }

        self.forward = f;
        self.right = r_proj;
        self.up = u_proj;

        // 4) Movimiento en función de la orientación local actualizada
        let mut move_dir = Vector3::new(0.0, 0.0, 0.0);
        if window.is_key_down(KeyboardKey::KEY_W) { move_dir -= self.forward; }
        if window.is_key_down(KeyboardKey::KEY_S) { move_dir += self.forward; }
        if window.is_key_down(KeyboardKey::KEY_D) { move_dir += self.right; }
        if window.is_key_down(KeyboardKey::KEY_A) { move_dir -= self.right; }
        if window.is_key_down(KeyboardKey::KEY_SPACE)      { move_dir += self.up; }
        if window.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) { move_dir -= self.up; }

        if move_dir.length() > 0.0 {
            self.translation += move_dir.normalized() * (speed * dt);
        }

        // 5) Reconstruir ángulos Euler a partir de la base (solo para el modelo)
        //    forward = (sin(yaw)*cos(pitch), -sin(pitch), cos(yaw)*cos(pitch))
        // => pitch = -asin(f.y)
        //    yaw   = atan2(f.x, f.z)
        let mut pitch = -self.forward.y.asin();
        let yaw = self.forward.x.atan2(self.forward.z);

        // Base sin roll para este forward
        let world_up = Vector3::new(0.0, 1.0, 0.0);
        let mut right0 = world_up.cross(self.forward);
        if right0.length() == 0.0 {
            right0 = Vector3::new(1.0, 0.0, 0.0);
        } else {
            right0 = right0.normalized();
        }
        let mut up0 = self.forward.cross(right0);
        if up0.length() == 0.0 {
            up0 = world_up;
        } else {
            up0 = up0.normalized();
        }

        // Con la base sin roll (right0, up0) y la base real (right, up),
        // el roll es el ángulo que rota right0 → right alrededor de forward.
        let cos_roll = self.right.dot(right0).clamp(-1.0, 1.0);
        let sin_roll = -self.up.dot(right0).clamp(-1.0, 1.0);
        let roll = sin_roll.atan2(cos_roll);

        self.rotation.x = pitch;
        self.rotation.y = yaw;
        self.rotation.z = roll;

        // Devolver up y right locales actualizados (para la cámara)
        (self.up, self.right)
    }
}

#[derive(Clone)]
pub enum Motion {
    Static,
    Orbit { center: Vector3, radius: f32, angular_speed: f32, phase: f32 }, // world-center orbit
    OrbitAround { parent: &'static str, radius: f32, angular_speed: f32, phase: f32 }, // orbit around entity
}