use raylib::prelude::*;
use crate::fragment::Fragment;
use crate::uniforms::Uniforms;

#[derive(Clone)]
pub enum VertexShader {
    Identity,
    SolarFlare,
}

#[inline]
fn dot3(a: Vector3, b: Vector3) -> f32 { a.x*b.x + a.y*b.y + a.z*b.z }

#[inline]
fn fract(x: f32) -> f32 { x - x.floor() }

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 { a + t * (b - a) }

#[inline]
fn fade(t: f32) -> f32 { t*t*t*(t*(t*6.0 - 15.0) + 10.0) }

#[inline]
fn hash3(p: Vector3) -> f32 {
    let n = dot3(p, Vector3::new(127.1, 311.7, 74.7));
    fract((n.sin() * 43758.5453).sin() * 143758.5453)
}

fn value_noise3(mut p: Vector3) -> f32 {
    let i = Vector3::new(p.x.floor(), p.y.floor(), p.z.floor());
    let f = Vector3::new(p.x - i.x, p.y - i.y, p.z - i.z);

    let n000 = hash3(i + Vector3::new(0.0,0.0,0.0));
    let n100 = hash3(i + Vector3::new(1.0,0.0,0.0));
    let n010 = hash3(i + Vector3::new(0.0,1.0,0.0));
    let n110 = hash3(i + Vector3::new(1.0,1.0,0.0));
    let n001 = hash3(i + Vector3::new(0.0,0.0,1.0));
    let n101 = hash3(i + Vector3::new(1.0,0.0,1.0));
    let n011 = hash3(i + Vector3::new(0.0,1.0,1.0));
    let n111 = hash3(i + Vector3::new(1.0,1.0,1.0));

    let u = Vector3::new(fade(f.x), fade(f.y), fade(f.z));

    let nx00 = lerp(n000, n100, u.x);
    let nx10 = lerp(n010, n110, u.x);
    let nx01 = lerp(n001, n101, u.x);
    let nx11 = lerp(n011, n111, u.x);

    let nxy0 = lerp(nx00, nx10, u.y);
    let nxy1 = lerp(nx01, nx11, u.y);

    lerp(nxy0, nxy1, u.z)
}

fn fbm(mut p: Vector3, octaves: i32, lacunarity: f32, gain: f32) -> f32 {
    let mut amp = 0.5;
    let mut freq = 1.0;
    let mut sum = 0.0;
    for _ in 0..octaves {
        sum += amp * value_noise3(Vector3::new(p.x*freq, p.y*freq, p.z*freq));
        freq *= lacunarity;
        amp *= gain;
    }
    sum
}

fn temperature_to_rgb(t: f32) -> Vector3 {
    // t in [0,1]: 0 = red/orange, 1 = white/blue
    // simple 3-point gradient: red -> yellow -> white
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        // red(1,0.2,0) to yellow(1,1,0)
        let k = t / 0.5;
        Vector3::new(1.0, lerp(0.2, 1.0, k), 0.0)
    } else {
        // yellow(1,1,0) to white(1,1,1) with slight blue tint
        let k = (t-0.5)/0.5;
        Vector3::new(1.0, 1.0, lerp(0.0, 0.3, k))
    }
}

pub fn apply_vertex_shader(v: Vector3, shader: &VertexShader, time: f32) -> Vector3 {
    match shader {
        VertexShader::Identity => v,
        VertexShader::SolarFlare => {
            // Displace along pseudo-normal (normalized position) with animated FBM
            let dir = if v.length() > 0.0 { v.normalized() } else { Vector3::new(0.0,0.0,1.0) };
            let p = Vector3::new(v.x*0.25, v.y*0.25, v.z*0.25 + time*0.2);
            let n = fbm(p, 4, 2.0, 0.5);
            let flare = (n*2.0 - 1.0) * 0.35; // amplitude in object units
            v + dir * flare
        }
    }
}

pub fn fragment_shader(fragment: &Fragment, u: &Uniforms) -> Vector3 {
    // Use object-space direction for stable texturing on the sphere surface
    let mut dir = fragment.obj_position;
    let len = (dir.x*dir.x + dir.y*dir.y + dir.z*dir.z).sqrt();
    if len > 0.0 { dir = Vector3::new(dir.x/len, dir.y/len, dir.z/len); }

    // FBM turbulence driven by object-space, time-cycled
    let tloop = (u.time % 8.0) / 8.0;
    let p3 = Vector3::new(dir.x*3.0, dir.y*3.0, tloop*8.0);
    let turb = fbm(p3, 5, 2.0, 0.55);

    // Core intensity based on how close to the disc center it projects (approx with dir.z)
    // dir.z ~ facing viewer if camera looks down -Z; use abs to be camera-agnostic
    let facing = dir.z.abs();
    let base_core = facing.clamp(0.0, 1.0);

    // User controls: temp in [0,1], intensity scaler ~ [0,2]
    let intensity = ((base_core * 0.7 + turb * 0.6) * u.intensity).clamp(0.0, 1.0);

    // Temperature affects gradient selection
    let color_base = temperature_to_rgb(((intensity + u.temp*0.8)*0.7).clamp(0.0,1.0));

    // Emission spikes add energetic flicker
    let spikes = (value_noise3(Vector3::new(dir.x*10.0 + u.time*1.7, dir.y*10.0 - u.time*1.3, u.time*0.5))*2.0-1.0).abs();
    let emission = (0.6*intensity + 0.8*spikes).clamp(0.0, 1.5);

    Vector3::new(
        (color_base.x * emission).clamp(0.0, 1.0),
        (color_base.y * emission).clamp(0.0, 1.0),
        (color_base.z * emission).clamp(0.0, 1.0),
    )
}