// main.rs
#![allow(unused_imports)]
#![allow(dead_code)]
#[inline]
fn rotate_y(v: Vector3, ang: f32) -> Vector3 {
    let (s, c) = ang.sin_cos();
    Vector3::new(c*v.x + 0.0*v.y + -s*v.z, v.y, s*v.x + 0.0*v.y + c*v.z)
}

use raylib::prelude::*;
use std::f32::consts::PI;
use std::time::Instant;

mod framebuffer;
mod camera;
mod matrix;
mod triangle;
mod fragment;
mod light;
mod entity;
mod shaders;
mod obj;

mod uniforms;
mod procedural;
use camera::Camera;
use entity::{Entity, Motion};
use framebuffer::Framebuffer;
use light::Light;
use uniforms::Uniforms;
use fragment::Fragment;
use obj::Obj;
use triangle::triangle;
use crate::{matrix::*, procedural::*, uniforms::*, shaders::*};

fn transform(
    vertex: Vector3,
    translation: Vector3,
    scale: f32,
    rotation: Vector3,
    view: &Matrix,
    projection: &Matrix,
    viewport: &Matrix,
) -> Vector3 {
    let model : Matrix = create_model_matrix(translation, scale, rotation);
    let vertex4 = Vector4::new(vertex.x, vertex.y, vertex.z, 1.0);

    let world_transform = multiply_matrix_vector4(&model, &vertex4);
    let view_transform = multiply_matrix_vector4(view, &world_transform);
    let projection_transform = multiply_matrix_vector4(projection, &view_transform);

    // División por w (NDC)
    let ndc = Vector4::new(
        projection_transform.x / projection_transform.w,
        projection_transform.y / projection_transform.w,
        projection_transform.z / projection_transform.w,
        1.0
    );

    // Viewport una sola vez (x,y), pero mantenemos depth en NDC [-1,1] para el Z-buffer
    let screen = multiply_matrix_vector4(viewport, &ndc);
    Vector3::new(screen.x, screen.y, ndc.z)
}

pub fn render(
    framebuffer: &mut Framebuffer,
    translation: Vector3,
    scale: f32,
    rotation: Vector3,
    vertex_array: &[Vector3],
    vshader: &VertexShader,
    view: &Matrix,
    projection: &Matrix,
    viewport: &Matrix,
    time: f32,
    resolution: Vector2,
    temp: f32,
    intensity: f32,
) {
    let light = Light::new(Vector3::new(0.0, 10.0, 0.0));
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    let mut obj_vertices_after_vs = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        let v_obj = apply_vertex_shader(*vertex, vshader, time);
        obj_vertices_after_vs.push(v_obj);
        let transformed = transform(v_obj, translation, scale, rotation, view, projection, viewport);
        transformed_vertices.push(transformed);
    }

    // Primitive Assembly Stage
    let mut triangles = Vec::new();
    let mut obj_tris = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
            obj_tris.push([
                obj_vertices_after_vs[i].clone(),
                obj_vertices_after_vs[i + 1].clone(),
                obj_vertices_after_vs[i + 2].clone(),
            ]);
        }
    }

    // Rasterization Stage
    let mut fragments = Vec::new();
    for (tri, obj_tri) in triangles.iter().zip(obj_tris.iter()) {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2], &obj_tri[0], &obj_tri[1], &obj_tri[2], &light));
    }
    
    let uniforms = Uniforms {
        time,
        resolution,
        temp,
        intensity,
    };

    // Fragment Processing Stage
    for fragment in fragments {
        let final_rgb = fragment_shader(&fragment, &uniforms);
        let out = vec3_to_color(final_rgb);
        framebuffer.set_current_color(out);
        framebuffer.set_pixel(
            fragment.position.x as u32,
            fragment.position.y as u32,
            fragment.depth
        );
    }

}

fn main() {
    let window_width = 1300;
    let window_height = 600;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Wireframe")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let projection = create_projection_matrix(PI/3.0, window_width as f32 / window_height as f32, 0.5, 100.0);
    let viewport = create_viewport_matrix(0.0, 0.0, window_width as f32, window_height as f32);

    let mut framebuffer = Framebuffer::new(window_width as u32, window_height as u32, Color::BLACK);
    framebuffer.set_background_color(Color::new(4, 12, 36, 255));

    let ship_obj = Obj::load("nave.obj").unwrap_or_else(|_| Obj::load("sphere.obj").expect("Failed to load any mesh"));
    let ship_vertices = ship_obj.get_vertex_array();

    let mut temp_control: f32 = 0.5;      // 0 (rojo) … 1 (blanco/azulado)
    let mut intensity_control: f32 = 1.0; // 1 = normal, >1 más brillante

    // --- Scene entities ---
    let mut entities: Vec<Entity> = vec![
        // The ship we will follow
        Entity {
            name: "ship",
            translation: Vector3::new(3.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: 1.0,
            motion: Motion::Static,
            vertices: ship_vertices.clone(),
            vshader: VertexShader::Identity,
            spin: Vector3::new(0.0, 0.0, 0.0),
            face_tangent: false,
        },

        Entity {
            name: "sun",
            translation: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: 1.0,
            motion: Motion::Orbit { 
                center: Vector3::new(0.0, 0.0, 0.0), radius: 10.0, angular_speed: 0.8, phase: 0.0 
            },
            vertices: generate_uv_sphere(3.0, 24, 32),
            vshader: VertexShader::SolarFlare,
            spin: Vector3::new(0.0, 1.0, 0.0),
            face_tangent: false,
        },
    ];


    let mut camera = Camera::new(
        Vector3::new(0.0, 0.0, 30.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    );

    let start_time = Instant::now();

    while !window.window_should_close() {
        framebuffer.clear();
        camera.process_input(&window);

        if window.is_key_down(KeyboardKey::KEY_RIGHT) { temp_control += 0.3 * window.get_frame_time(); }
        if window.is_key_down(KeyboardKey::KEY_LEFT)  { temp_control -= 0.3 * window.get_frame_time(); }
        if window.is_key_down(KeyboardKey::KEY_UP)    { intensity_control += 0.5 * window.get_frame_time(); }
        if window.is_key_down(KeyboardKey::KEY_DOWN)  { intensity_control -= 0.5 * window.get_frame_time(); }
        temp_control = temp_control.clamp(0.0, 1.0);
        intensity_control = intensity_control.clamp(0.2, 2.0);

        // Global time and resolution
        let time = start_time.elapsed().as_secs_f32();
        let resolution = Vector2::new(window_width as f32, window_height as f32);

        // --- Update entity motions ---
        use std::collections::HashMap;
        let index_by_name: HashMap<&'static str, usize> = entities.iter().enumerate().map(|(i,e)| (e.name, i)).collect();

        // Pass 1: update world-centered orbits and statics
        for i in 0..entities.len() {
            match entities[i].motion {
                Motion::Static => { /* no-op */ }
                Motion::Orbit { center, radius, angular_speed, phase } => {
                    let theta = phase + angular_speed * time;
                    entities[i].translation.x = center.x + radius * theta.cos();
                    entities[i].translation.z = center.z + radius * theta.sin();
                    entities[i].translation.y = center.y;
                    // entities[i].rotation.y = -theta; // removed
                }
                Motion::OrbitAround { .. } => { /* defer to pass 2 */ }
            }
        }
        
        // Pass 2: update children that orbit around a parent (world-axes offset around parent's position)
        for i in 0..entities.len() {
            if let Motion::OrbitAround { parent, radius, angular_speed, phase } = entities[i].motion.clone() {
                if let Some(&pi) = index_by_name.get(parent) {
                    let parent_pos = entities[pi].translation;
                    let theta = phase + angular_speed * time;

                    if radius == 0.0 {
                        // Keep centered on parent; allow spin-in-place via rotation if desired
                        entities[i].translation = parent_pos;
                        // entities[i].rotation.y = -theta; // removed
                    } else {
                        // Orbit around parent in world axes (no coupling to parent's heading)
                        let world_offset = Vector3::new(radius * theta.cos(), 0.0, radius * theta.sin());
                        entities[i].translation = Vector3::new(
                            parent_pos.x + world_offset.x,
                            parent_pos.y + world_offset.y,
                            parent_pos.z + world_offset.z,
                        );
                        // entities[i].rotation.y = -theta; // removed
                    }
                }
            }
        }

        // --- Follow camera: lock target to sun position ---
        if let Some(ship) = entities.iter().find(|ent| ent.name == "ship") {
            camera.set_target(ship.translation);
        }

        let view = camera.get_view_matrix();

        // --- Render all entities ---
        for e in &entities {

            let mut rot = e.rotation;

            // Add tangent-facing yaw from orbital motion if requested
            if e.face_tangent {
                match e.motion {
                    Motion::Orbit { angular_speed, phase, .. } => {
                        let theta = phase + angular_speed * time;
                        rot.y += -theta;
                    }
                    Motion::OrbitAround { angular_speed, phase, .. } => {
                        let theta = phase + angular_speed * time;
                        rot.y += -theta;
                    }
                    Motion::Static => {}
                }
            }

            rot.x += e.spin.x * time;
            rot.y += e.spin.y * time;
            rot.z += e.spin.z * time;

            render(
                &mut framebuffer,
                e.translation,
                e.scale,
                rot,
                &e.vertices,
                &e.vshader,
                &view,
                &projection,
                &viewport,
                time,
                resolution,
                temp_control,
                intensity_control,

            );
        }

        framebuffer.swap_buffers(&mut window, &raylib_thread);
    }
}
