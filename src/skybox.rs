use std::{f32::consts::PI, time::Instant};

use raylib::prelude::*;
use crate::{framebuffer::Framebuffer, light::Light, matrix::{create_projection_matrix, create_viewport_matrix}, procedural::generate_uv_sphere, skybox, transform, triangle::{triangle, triangle_sky}, uniforms::{color_to_vec3, vec3_to_color}};
// --- 3D Skybox / Nebula / Stars ---
fn sky_hash31(p: Vector3) -> f32 {
    let n = p.x*157.0 + p.y*113.0 + p.z*427.0;
    ( (n.sin()*43758.5453).fract() * (n.cos()*12345.6789).fract() ).fract()
}

fn sky_noise3(p: Vector3) -> f32 {
    let i = Vector3::new(p.x.floor(), p.y.floor(), p.z.floor());
    let f = Vector3::new(p.x-i.x, p.y-i.y, p.z-i.z);

    let c000 = sky_hash31(i);
    let c100 = sky_hash31(i + Vector3::new(1.0,0.0,0.0));
    let c010 = sky_hash31(i + Vector3::new(0.0,1.0,0.0));
    let c110 = sky_hash31(i + Vector3::new(1.0,1.0,0.0));
    let c001 = sky_hash31(i + Vector3::new(0.0,0.0,1.0));
    let c101 = sky_hash31(i + Vector3::new(1.0,0.0,1.0));
    let c011 = sky_hash31(i + Vector3::new(0.0,1.0,1.0));
    let c111 = sky_hash31(i + Vector3::new(1.0,1.0,1.0));

    let fu = f.x*f.x*(3.0 - 2.0*f.x);
    let fv = f.y*f.y*(3.0 - 2.0*f.y);
    let fw = f.z*f.z*(3.0 - 2.0*f.z);

    let x00 = c000 + (c100-c000)*fu;
    let x10 = c010 + (c110-c010)*fu;
    let x01 = c001 + (c101-c001)*fu;
    let x11 = c011 + (c111-c011)*fu;

    let y0 = x00 + (x10-x00)*fv;
    let y1 = x01 + (x11-x01)*fv;

    y0 + (y1-y0)*fw
}

fn fbm3(mut p: Vector3, oct: i32, lac: f32, gain: f32) -> f32 {
    let mut amp = 0.5;
    let mut freq = 1.0;
    let mut sum = 0.0;
    for _ in 0..oct {
        sum += amp * sky_noise3(p*freq);
        freq *= lac;
        amp *= gain;
    }
    sum
}

fn sample_sky(dir: Vector3, time: f32) -> Color {
    let d = dir.normalized();

    // Nebulosa 3D
    let neb = fbm3(d*2.5, 5, 2.2, 0.55);
    let neb2 = fbm3(d*7.0 + Vector3::new(0.0, time*0.03, 0.0), 4, 2.0, 0.5);
    let neb_mix = (neb*0.6 + neb2*0.4).clamp(0.0, 1.0);

    let base_r = (5.0 + neb_mix*40.0) as u8;
    let base_g = (8.0 + neb_mix*60.0) as u8;
    let base_b = (20.0 + neb_mix*140.0) as u8;
    let mut col = Color::new(base_r, base_g, base_b, 255);

    // Estrellas puntuales
    let star_noise = sky_hash31(d * 120.0);
    if star_noise > 0.9985 {
        let a = ((star_noise - 0.9985)/0.0015).clamp(0.0,1.0);
        let b = (210.0 + 45.0*a) as u8;
        col = Color::new(b, b, (b as f32*1.1).min(255.0) as u8, 255);
    } else if star_noise > 0.995 {
        let b = (160.0 + (star_noise-0.995)/0.005*80.0) as u8;
        col = Color::new(b, b, (b as f32*1.05).min(255.0) as u8, 255);
    }

    col
}

pub struct Skybox{
    pub vertices: Vec<Vector3>,
    pub colors: Vec<Vector3>
}
impl Skybox {
    pub fn new() -> Self { 
        let vertices = generate_uv_sphere(10000.0, 100, 100);
        let mut colors = Vec::new();
        let time = Instant::now().elapsed().as_secs_f32();
        for vtx in vertices.clone() {
            colors.push(color_to_vec3(sample_sky(vtx, time)));
        }
        Skybox {
            vertices,
            colors,
        }
    }
}

pub fn draw_sky_sphere(framebuffer: &mut Framebuffer, skybox: &Skybox, view: &Matrix) {
    let projection = create_projection_matrix(PI/3.0, framebuffer.height as f32 / framebuffer.height as f32, 0.5, 100.0);
    let viewport = create_viewport_matrix(0.0, 0.0, framebuffer.width as f32, framebuffer.width as f32);
    let mut transformed_vertices = Vec::with_capacity(skybox.vertices.len());
    for (vtx, col) in skybox.vertices.iter().zip(skybox.colors.iter()) {
        let tv = transform(vtx.clone(), Vector3::new(0.0, 0.0, 0.0), 1.0, Vector3::new(0.0, 0.0, 0.0), &view, &projection, &viewport);
        transformed_vertices.push(tv);

        
    }
     // Primitive Assembly Stage
    let mut fragcols = Vec::new();
    let mut triangles = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 >= transformed_vertices.len() {
            break;
        }

        if let (Some(v0), Some(v1), Some(v2)) = (
            transformed_vertices[i],
            transformed_vertices[i + 1],
            transformed_vertices[i + 2],
        ) {
            triangles.push([v0, v1, v2]);
            fragcols.push([skybox.colors[i], skybox.colors[i+1], skybox.colors[i+2]]);
        }
    }

    let light = Light::new(Vector3::new(0.0, 0.0, 0.0));

    // Rasterization Stage
    let mut fragments = Vec::new();
    for (tri, obj_tri) in triangles.iter().zip(fragcols.iter()) {
        fragments.extend(triangle_sky(&tri[0], &tri[1], &tri[2], &obj_tri[0], &obj_tri[1], &obj_tri[2]));
    }
    for fragment in fragments {
        framebuffer.set_current_color(vec3_to_color(fragment.color));
        framebuffer.set_pixel(
            fragment.position.x as u32,
            fragment.position.y as u32,
            fragment.depth
        );
    }
}