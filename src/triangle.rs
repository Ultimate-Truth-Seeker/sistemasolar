// src/triangle.rs
use raylib::prelude::*;
use crate::fragment::Fragment;
use crate::light::Light;
use crate::uniforms::color_to_vec3;

fn barycentric_coordinates(p_x: f32, p_y: f32, a: &Vector3, b: &Vector3, c: &Vector3)  -> (f32, f32, f32) {
    let a_x = a.x;
    let b_x = b.x;
    let c_x = c.x;
    let a_y = a.y;
    let b_y = b.y;
    let c_y = c.y;

    let area = (b_y - c_y) * (a_x - c_x) + (c_x - b_x) * (a_y - c_y);

    if area.abs() < 1e-10  {
        return (-1.0, -1.0, -1.0);
    }
    
    let w = ((b_y - c_y) * (p_x - c_x) + (c_x - b_x) * (p_y - c_y)) / area;
    let v = ((c_y - a_y) * (p_x - c_x) + (a_x - c_x) * (p_y - c_y)) / area;
    let u = 1.0 - w - v;

    (w, v, u)
}

pub fn triangle(v1: &Vector3, v2: &Vector3, v3: &Vector3, obj1: &Vector3, obj2: &Vector3, obj3: &Vector3, light: &Light) -> Vec<Fragment> {
    let mut fragments: Vec<Fragment> = Vec::new();

    let a_x = v1.x;
    let b_x = v2.x;
    let c_x = v3.x;
    let a_y = v1.y;
    let b_y = v2.y;
    let c_y = v3.y;

    let min_x = a_x.min(b_x).min(c_x).floor() as i32;
    let min_y = a_y.min(b_y).min(c_y).floor() as i32;

    let max_x = a_x.max(b_x).max(c_x).ceil() as i32;
    let max_y = a_y.max(b_y).max(c_y).ceil() as i32;

    let color_a = Vector3::new(1.0, 0.0, 0.0);
    let color_b = Vector3::new(0.0, 1.0, 0.0);
    let color_c = Vector3::new(0.0, 0.0, 1.0);

    let light = light.position.normalized();
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let (w, v, u) = barycentric_coordinates(x  as f32, y as f32, v1, v2, v3);

            let depth = v1.z*w + v2.z*v + v3.z*u;

            let intensity = v1.dot(light).max(0.0);
            let color = Vector3::new(1.0, 1.0, 1.0);//color_a*w +color_b*v + color_c*u;//
            let final_color = color*intensity;

            let ox = obj1.x*w + obj2.x*v + obj3.x*u;
            let oy = obj1.y*w + obj2.y*v + obj3.y*u;
            let oz = obj1.z*w + obj2.z*v + obj3.z*u;
            let obj_pos = Vector3::new(ox, oy, oz);

            if w >= 0.0 && v >= 0.0 && u >= 0.0 {
                fragments.push(Fragment::new(
                    x as f32,
                    y as f32,
                    final_color,
                    depth,
                    obj_pos,
                ));
            }
        }
    }


    fragments
}

pub fn triangle_sky(v1: &Vector3, v2: &Vector3, v3: &Vector3, obj1: &Vector3, obj2: &Vector3, obj3: &Vector3) -> Vec<Fragment> {
    let mut fragments: Vec<Fragment> = Vec::new();

    let a_x = v1.x;
    let b_x = v2.x;
    let c_x = v3.x;
    let a_y = v1.y;
    let b_y = v2.y;
    let c_y = v3.y;

    let min_x = a_x.min(b_x).min(c_x).floor() as i32;
    let min_y = a_y.min(b_y).min(c_y).floor() as i32;

    let max_x = a_x.max(b_x).max(c_x).ceil() as i32;
    let max_y = a_y.max(b_y).max(c_y).ceil() as i32;

    let color_a = obj1.clone();
    let color_b = obj2.clone();
    let color_c = obj3.clone();

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let (w, v, u) = barycentric_coordinates(x  as f32, y as f32, v1, v2, v3);

            let depth = v1.z*w + v2.z*v + v3.z*u;
            let color = color_a*w +color_b*v + color_c*u;//
            let final_color = color;

            let ox = v1.x*w + v2.x*v + v3.x*u;
            let oy = v1.y*w + v2.y*v + v3.y*u;
            let oz = v2.z*w + v2.z*v + v3.z*u;
            let obj_pos = Vector3::new(ox, oy, oz);

            if w >= 0.0 && v >= 0.0 && u >= 0.0 {
                fragments.push(Fragment::new(
                    x as f32,
                    y as f32,
                    final_color,
                    depth,
                    obj_pos,
                ));
            }
        }
    }


    fragments
}