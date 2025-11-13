use std::f32::consts::PI;

use raylib::prelude::*;
use crate::fragment::Fragment;
use crate::uniforms::Uniforms;

#[derive(Clone)]
pub enum VertexShader {
    Identity,
    SolarFlare,
    DisplacePlanarY  { amp: f32, freq: f32, octaves: u32, lacunarity: f32, gain: f32, time_amp: f32 },
}

#[derive(Clone)]
pub enum FragmentShader {
    Star,
    Solid { color: Vector3 },
    Rocky { color: Vector3 },
    Strips { angle: f32 },
    AlienShip
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
        },
        VertexShader::DisplacePlanarY { amp, freq, octaves, lacunarity, gain, time_amp } => {
            // For rings/planes, displace along +Y using FBM in XZ
            let p = Vector3::new(v.x * *freq, 0.0, v.z * *freq) + Vector3::new(0.0, 0.0, time * *time_amp);
            let h = crate::procedural::fbm3(p, *octaves, *lacunarity, *gain); // ~[-1,1]
            let disp = *amp * h;
            Vector3::new(v.x, v.y + disp, v.z)
        }
    }
}

pub fn fragment_shader(fragment: &Fragment, u: &Uniforms, shader: &FragmentShader) -> Vector3 {
    match shader {
        FragmentShader::Star => {
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
        },
        FragmentShader::Solid { color } => {
            let pos = fragment.position;
            let base_color = color.clone();
            let time = u.time;

            let angle = pos.x.atan2(pos.z);// + time;
            let hue = (angle / 2.0 * PI) % 1.0;

            let r = (hue * 5.0).sin().abs();
            let g = (hue * 5.0).sin().abs(); 
            let b = (hue * 5.0).sin().abs();

            let pattern_color = Vector3::new(r, g, b);

            base_color * 0.5 + pattern_color * 0.5

        },
        FragmentShader::Rocky { color } => {
            let mut p = fragment.obj_position;
            let len = (p.x*p.x + p.y*p.y + p.z*p.z).sqrt();
            if len > 0.0 {
                p = Vector3::new(p.x/len, p.y/len, p.z/len); // dirección en la esfera
            }

            // Base de roca: fbm de baja frecuencia
            let base = fbm(p * 4.0, 4, 2.0, 0.5);  // 0..~1
            let base2 = fbm(p * 12.0, 3, 2.4, 0.55);
            let rocky = (base*0.7 + base2*0.3).clamp(0.0, 1.0);

            // Color rocoso (marrón/gris)
            let albedo = Vector3::new(
              //  0.25 + 0.25*rocky, // R
                //0.2  + 0.2*rocky,  // G
                //0.18 + 0.15*rocky, // B
                color.x + 0.25*rocky,
                color.y +0.2*rocky,
                color.z + 0.15*rocky,
            );

            // Cráteres: patrón de “huecos” oscuros fijos en el objeto
            // Usamos un ruido de alta frecuencia y lo umbralizamos
            let crater_noise = fbm(p * 16.0, 3, 2.2, 0.5);
            let mut crater_mask = (crater_noise - 0.55) * 8.0; // valores por debajo generan hoyos
            crater_mask = crater_mask.clamp(0.0, 1.0);
            // invertimos: 1 = superficie, 0 = cráter
            let crater = 1.0 - crater_mask;

            let crater_dark = 0.35; // qué tan oscuros son los cráteres
            let color = Vector3::new(
                albedo.x * (crater_dark + (1.0-crater_dark)*crater),
                albedo.y * (crater_dark + (1.0-crater_dark)*crater),
                albedo.z * (crater_dark + (1.0-crater_dark)*crater),
            );

            // Un toquecito de iluminación básica tipo lambert con el Sol en el origen:
            let light_dir = Vector3::new(0.0, 0.0, 0.0) - fragment.obj_position;
            let l_len = (light_dir.x*light_dir.x + light_dir.y*light_dir.y + light_dir.z*light_dir.z).sqrt();
            let ndotl = if l_len > 0.0 {
                let l = Vector3::new(light_dir.x/l_len, light_dir.y/l_len, light_dir.z/l_len);
                let n = p;
                (n.x*l.x + n.y*l.y + n.z*l.z).max(0.0)
            } else {
                1.0
            };

            let diffuse = 0.65 + 0.35*ndotl;

            Vector3::new(
                (color.x * diffuse).clamp(0.0, 1.0),
                (color.y * diffuse).clamp(0.0, 1.0),
                (color.z * diffuse).clamp(0.0, 1.0),
            )
        },
        FragmentShader::Strips { angle } => {
            let mut p = fragment.obj_position;
            let len = (p.x*p.x + p.y*p.y + p.z*p.z).sqrt();
            if len > 0.0 {
                p = Vector3::new(p.x/len, p.y/len, p.z/len);
            }

            // latitud en [-1,1]
            let lat = p.y;

            // Distorsión de las bandas por ruido (animado)
            let t = u.time * 0.15;
            let warp = fbm(
                Vector3::new(p.x*6.0, p.y*6.0, p.z*6.0 + t),
                4,
                2.1,
                0.5,
            );
            let lat_warped = lat + (warp - 0.5) * 0.25; // distorsión suave

            // Periodicidad de bandas: usamos varias “zonas”
            // stripe = sin(k * lat_warped) → alterna claro/oscuro
            let k = 14.0; // número de bandas
            let stripe_val = (k * lat_warped).sin();

            // Mapear a 0..1 y hacer más duras las franjas
            let bands = (stripe_val * 1.2).tanh(); // transiciones suavizadas pero no tan lisas
            let bands01 = (bands * 0.5 + 0.5).clamp(0.0, 1.0);

            // Dos colores base tipo Júpiter
            let band_light = Vector3::new(0.95, 0.9, 0.78);
            let band_dark  = Vector3::new(0.82, 0.6, 0.45);

            let mut color = Vector3::new(
                band_dark.x + (band_light.x - band_dark.x) * bands01,
                band_dark.y + (band_light.y - band_dark.y) * bands01,
                band_dark.z + (band_light.z - band_dark.z) * bands01,
            );

            // Añadir turbulencia en “nubes” usando ruido
            let clouds = fbm(Vector3::new(p.x*10.0 + t*0.7, p.y*18.0, p.z*10.0 - t*0.5), 5, 2.1, 0.5);
            let clouds_mask = (clouds - 0.4).max(0.0) * 1.8;
            let clouds_mask = clouds_mask.clamp(0.0, 1.0);

            let cloud_tint = Vector3::new(1.0, 0.98, 0.95);
            color = Vector3::new(
                color.x + (cloud_tint.x - color.x) * clouds_mask,
                color.y + (cloud_tint.y - color.y) * clouds_mask,
                color.z + (cloud_tint.z - color.z) * clouds_mask,
            );

            // Opcional: pequeñas manchas (spots) de tormentas, fijas o casi fijas
            let spots = fbm(Vector3::new(p.x*20.0, p.y*20.0, p.z*20.0), 3, 2.0, 0.5);
            let mut spots_mask = (spots - 0.75) * 6.0;
            spots_mask = spots_mask.clamp(0.0, 1.0);
            let spot_color = Vector3::new(0.8, 0.4, 0.2);

            color = Vector3::new(
                color.x*(1.0-spots_mask) + spot_color.x*spots_mask,
                color.y*(1.0-spots_mask) + spot_color.y*spots_mask,
                color.z*(1.0-spots_mask) + spot_color.z*spots_mask,
            );

            // Simple iluminación desde el sol en el origen
            let light_dir = Vector3::new(0.0, 0.0, 0.0) - fragment.obj_position;
            let l_len = (light_dir.x*light_dir.x + light_dir.y*light_dir.y + light_dir.z*light_dir.z).sqrt();
            let ndotl = if l_len > 0.0 {
                let l = Vector3::new(light_dir.x/l_len, light_dir.y/l_len, light_dir.z/l_len);
                let n = p;
                (n.x*l.x + n.y*l.y + n.z*l.z).max(0.0)
            } else { 1.0 };

            let diffuse = 0.8 + 0.2*ndotl;

            Vector3::new(
                (color.x * diffuse).clamp(0.0, 1.0),
                (color.y * diffuse).clamp(0.0, 1.0),
                (color.z * diffuse).clamp(0.0, 1.0),
            )
        },
        FragmentShader::AlienShip => {
            let mut p = fragment.obj_position;
            let len = (p.x*p.x + p.y*p.y + p.z*p.z).sqrt();
            if len > 0.0 {
                p = Vector3::new(p.x/len, p.y/len, p.z/len);
            }
            // latitud en [-1,1]
            let lat = p.y;
            if lat >= -0.5 && lat <= 0.27 || lat >= 0.43{
                Vector3::new(0.7, 0.7, 0.7)
            } else {
                Vector3::new(0.0, 1.0, 0.0)
            }
        }
    }
}