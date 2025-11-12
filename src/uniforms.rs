// uniforms.rs (si quieres en un archivo aparte) o al inicio de tu shader.rs
use raylib::prelude::*;

pub struct Uniforms {
    pub time: f32,         // segundos
    pub resolution: Vector2, // tamaño ventana en píxeles
    pub temp: f32,
    pub intensity: f32,
}

// Convierte Color (0..255) a vec3 0..1
pub fn color_to_vec3(c: Color) -> Vector3 {
    Vector3::new(
        c.r as f32 / 255.0,
        c.g as f32 / 255.0,
        c.b as f32 / 255.0,
    )
}

// Convierte vec3 0..1 a Color (0..255)
pub fn vec3_to_color(v: Vector3) -> Color {
    let r = (v.x.clamp(0.0, 1.0) * 255.0) as u8;
    let g = (v.y.clamp(0.0, 1.0) * 255.0) as u8;
    let b = (v.z.clamp(0.0, 1.0) * 255.0) as u8;
    Color::new(r, g, b, 255)
}