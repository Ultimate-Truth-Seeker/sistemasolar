// src/procedural.rs
use raylib::prelude::*;

/// Generate a UV-sphere (triangulated) without external models.
/// Returns a flat Vec of positions grouped in triangles (3-by-3).
pub fn generate_uv_sphere(radius: f32, lat_segments: usize, lon_segments: usize) -> Vec<Vector3> {
    let lat = lat_segments.max(3);
    let lon = lon_segments.max(3);
    let mut out: Vec<Vector3> = Vec::with_capacity(lat * lon * 6);

    for i in 0..lat {
        // v ranges from 0..1, phi from 0..PI
        let v0 = i as f32 / lat as f32;
        let v1 = (i + 1) as f32 / lat as f32;
        let phi0 = v0 * std::f32::consts::PI;
        let phi1 = v1 * std::f32::consts::PI;

        for j in 0..lon {
            // u ranges from 0..1, theta from 0..2PI
            let u0 = j as f32 / lon as f32;
            let u1 = (j + 1) as f32 / lon as f32;
            let theta0 = u0 * std::f32::consts::TAU;
            let theta1 = u1 * std::f32::consts::TAU;

            let p00 = sph(radius, phi0, theta0);
            let p01 = sph(radius, phi0, theta1);
            let p10 = sph(radius, phi1, theta0);
            let p11 = sph(radius, phi1, theta1);

            // Two triangles per quad (p00, p10, p11) and (p00, p11, p01)
            out.push(p00); out.push(p10); out.push(p11);
            out.push(p00); out.push(p11); out.push(p01);
        }
    }
    out
}

#[inline]
fn sph(r: f32, phi: f32, theta: f32) -> Vector3 {
    // Spherical coordinates: phi ∈ [0,PI] from north to south, theta ∈ [0,2PI]
    let sin_phi = phi.sin();
    Vector3::new(
        r * sin_phi * theta.cos(),
        r * phi.cos(),
        r * sin_phi * theta.sin(),
    )
}

/// Generate a flat ring (annulus) in the XZ plane centered at origin.
/// The ring thickness is [inner_radius, outer_radius].
/// Returns triangles (Vector3) in object space.
pub fn generate_ring(inner_radius: f32, outer_radius: f32, segments: usize) -> Vec<Vector3> {
    let n = segments.max(3);
    let mut out: Vec<Vector3> = Vec::with_capacity(n * 6);

    for i in 0..n {
        let t0 = i as f32 / n as f32;
        let t1 = (i + 1) as f32 / n as f32;
        let a0 = t0 * std::f32::consts::TAU;
        let a1 = t1 * std::f32::consts::TAU;

        let i0 = polar(inner_radius, a0);
        let i1 = polar(inner_radius, a1);
        let o0 = polar(outer_radius, a0);
        let o1 = polar(outer_radius, a1);

        // Quad as two triangles (o0, i0, i1) and (o0, i1, o1)
        out.push(o0); out.push(i0); out.push(i1);
        out.push(o0); out.push(i1); out.push(o1);
    }

    out
}

#[inline]
fn polar(r: f32, ang: f32) -> Vector3 {
    Vector3::new(r * ang.cos(), 0.0, r * ang.sin())
}

// --- 3D Value Noise + FBM (no external crates) ---
#[inline]
fn hash_u32(mut x: u32) -> u32 {
    // Thomas Wang mix
    x = x.wrapping_add(0x9E3779B9);
    x ^= x >> 15;
    x = x.wrapping_mul(0x85EBCA6B);
    x ^= x >> 13;
    x = x.wrapping_mul(0xC2B2AE35);
    x ^= x >> 16;
    x
}

#[inline]
fn lattice_rand(i: i32, j: i32, k: i32) -> f32 {
    let mut h = 1469598103u32; // FNV-like mix
    h ^= i as u32; h = h.wrapping_mul(16777619);
    h ^= j as u32; h = h.wrapping_mul(16777619);
    h ^= k as u32; h = h.wrapping_mul(16777619);
    let r = hash_u32(h);
    (r as f32) / (u32::MAX as f32) // 0..1
}

#[inline]
fn smoothstep(t: f32) -> f32 { t * t * (3.0 - 2.0 * t) }

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 { a + (b - a) * t }

/// 3D value noise in [0,1]
pub fn value_noise3(p: Vector3) -> f32 {
    let x0 = p.x.floor() as i32;
    let y0 = p.y.floor() as i32;
    let z0 = p.z.floor() as i32;
    let x1 = x0 + 1;
    let y1 = y0 + 1;
    let z1 = z0 + 1;

    let tx = smoothstep(p.x - x0 as f32);
    let ty = smoothstep(p.y - y0 as f32);
    let tz = smoothstep(p.z - z0 as f32);

    let c000 = lattice_rand(x0, y0, z0);
    let c100 = lattice_rand(x1, y0, z0);
    let c010 = lattice_rand(x0, y1, z0);
    let c110 = lattice_rand(x1, y1, z0);
    let c001 = lattice_rand(x0, y0, z1);
    let c101 = lattice_rand(x1, y0, z1);
    let c011 = lattice_rand(x0, y1, z1);
    let c111 = lattice_rand(x1, y1, z1);

    let x00 = lerp(c000, c100, tx);
    let x10 = lerp(c010, c110, tx);
    let x01 = lerp(c001, c101, tx);
    let x11 = lerp(c011, c111, tx);

    let y0v = lerp(x00, x10, ty);
    let y1v = lerp(x01, x11, ty);

    lerp(y0v, y1v, tz)
}

/// Fractal Brownian Motion (FBM) using value_noise3; returns ~[-1,1]
pub fn fbm3(mut p: Vector3, octaves: u32, lacunarity: f32, gain: f32) -> f32 {
    let mut amp = 0.5;
    let mut sum = 0.0;
    let mut total_amp = 0.0;
    for _ in 0..octaves {
        let n = value_noise3(p) * 2.0 - 1.0; // to [-1,1]
        sum += n * amp;
        total_amp += amp;
        p.x *= lacunarity; p.y *= lacunarity; p.z *= lacunarity;
        amp *= gain;
    }
    if total_amp > 0.0 { sum / total_amp } else { 0.0 }
}
