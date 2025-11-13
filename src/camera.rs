#![allow(dead_code)]

use raylib::prelude::*;
use crate::matrix::create_view_matrix;

/// Cámara sencilla que se “pega” a la nave.
/// No tiene yaw/pitch/roll propios: solo mantiene un offset relativo
/// respecto al plano local de la nave y lo traduce a coordenadas de mundo.
pub struct Camera {
    pub eye: Vector3,      // Posición de la cámara en mundo
    pub target: Vector3,   // Punto al que mira (en este caso, la nave)
    pub up: Vector3,       // Up en mundo (normalmente el up de la nave)
    pub right: Vector3,
    pub forward: Vector3,

    /// Dirección del offset en el sistema local de la nave: (right, up, back)
    /// Por ejemplo: (0, 0.5, 1.0) = un poco arriba y detrás de la nave.
    pub offset_dir_local: Vector3,

    /// Distancia de la cámara a la nave (escala del offset_dir_local).
    pub distance: f32,

    pub zoom_speed: f32,
}

impl Camera {
    /// Crea una cámara dada una posición inicial y un target.
    /// El offset local se inicializa proyectando la posición relativa sobre
    /// una base “genérica” (right=(1,0,0), up=(0,1,0), forward=(0,0,1)).
    pub fn new(initial_eye: Vector3, initial_target: Vector3) -> Self {
        let up = Vector3::new(0.0, 1.0, 0.0);

        let offset = initial_eye - initial_target;
        let distance = offset.length().max(0.001);

        // Base genérica para inicializar la dirección local del offset
        let right = Vector3::new(1.0, 0.0, 0.0);
        let up_basis = Vector3::new(0.0, 1.0, 0.0);
        let forward = Vector3::new(0.0, 0.0, 1.0);

        let rel_x = offset.dot(right);
        let rel_y = offset.dot(up_basis);
        let rel_z = -offset.dot(forward);

        let mut offset_dir_local = Vector3::new(rel_x, rel_y, rel_z);
        if offset_dir_local.length() > 0.0 {
            offset_dir_local = offset_dir_local / offset_dir_local.length();
        } else {
            offset_dir_local = Vector3::new(0.0, 0.0, 1.0);
        }

        Camera {
            eye: initial_eye,
            target: initial_target,
            up,
            right,
            forward,
            offset_dir_local,
            distance,
            zoom_speed: 0.5,
        }
    }

    /// Actualiza eye/target/up a partir de la orientación de la nave.
    /// ship_right y ship_up vienen en coordenadas de mundo, calculados por la nave.
    /// El eje forward se deduce como forward = normalize(right × up).
    pub fn follow_ship(&mut self, ship_pos: Vector3, ship_right: Vector3, ship_up: Vector3) {
        self.target = ship_pos;

        // Ortonormalizar base de la nave
        let mut r = ship_right;
        let mut u = ship_up;

        if r.length() == 0.0 {
            r = Vector3::new(1.0, 0.0, 0.0);
        } else {
            r = r / r.length();
        }
        if u.length() == 0.0 {
            u = Vector3::new(0.0, 1.0, 0.0);
        } else {
            u = u / u.length();
        }

        // forward = normalize(right × up)
        let mut f = r.cross(u);
        if f.length() == 0.0 {
            f = Vector3::new(0.0, 0.0, 1.0);
        } else {
            f = f / f.length();
        }

        // Dirección local del offset en base de la nave (right, up, back)
        let dir = {
            let mut d = self.offset_dir_local;
            if d.length() > 0.0 {
                d = d / d.length();
            }
            d
        };

        // Escalar por distancia actual
        let scaled = dir * self.distance;

        // Interpretar offset_local en la base (right, up, -forward)
        let world_offset = Vector3::new(
            r.x * scaled.x + u.x * scaled.y - f.x * scaled.z,
            r.y * scaled.x + u.y * scaled.y - f.y * scaled.z,
            r.z * scaled.x + u.z * scaled.y - f.z * scaled.z,
        );

        self.eye = ship_pos + world_offset;
        self.up = u;
        self.right = r;
        self.forward = f;
    }

    /// Zoom in: acercar cámara a la nave (reduce distance).
    pub fn zoom_in(&mut self) {
        self.distance -= self.zoom_speed;
        if self.distance < 0.5 {
            self.distance = 0.5;
        }
    }

    /// Zoom out: alejar cámara de la nave (incrementa distance).
    pub fn zoom_out(&mut self) {
        self.distance += self.zoom_speed;
    }

    /// View matrix para el rasterizador.
    pub fn get_view_matrix(&self) -> Matrix {
        create_view_matrix(self.eye, self.target, self.up)
    }
}