#![allow(dead_code)]

use raylib::prelude::*;

pub fn multiply_matrix_vector4(matrix: &Matrix, vector: &Vector4) -> Vector4 {
    Vector4::new(
        matrix.m0 * vector.x + matrix.m4 * vector.y + matrix.m8 * vector.z + matrix.m12 * vector.w,
        matrix.m1 * vector.x + matrix.m5 * vector.y + matrix.m9 * vector.z + matrix.m13 * vector.w,
        // This function manually multiplies a 4x4 matrix with a 4D vector (in homogeneous coordinates)
        matrix.m2 * vector.x + matrix.m6 * vector.y + matrix.m10 * vector.z + matrix.m14 * vector.w,
        matrix.m3 * vector.x + matrix.m7 * vector.y + matrix.m11 * vector.z + matrix.m15 * vector.w,
    )
}

/// Creates a 4x4 matrix from 16 float values, specified in traditional row-major order.
pub fn new_matrix4(
    // Row 0
    r0c0: f32, r0c1: f32, r0c2: f32, r0c3: f32,
    // Row 1
    r1c0: f32, r1c1: f32, r1c2: f32, r1c3: f32,
    // Row 2
    r2c0: f32, r2c1: f32, r2c2: f32, r2c3: f32,
    // Row 3
    r3c0: f32, r3c1: f32, r3c2: f32, r3c3: f32,
) -> Matrix {
    // Raylib's Matrix is column-major, so we transpose the row-major input.
    Matrix {
        m0: r0c0, m1: r1c0, m2: r2c0, m3: r3c0, // Column 0
        m4: r0c1, m5: r1c1, m6: r2c1, m7: r3c1, // Column 1
        m8: r0c2, m9: r1c2, m10: r2c2, m11: r3c2, // Column 2
        m12: r0c3, m13: r1c3, m14: r2c3, m15: r3c3, // Column 3
    }
}

/// Creates a 4x4 transformation matrix from a 3x3 matrix, specified in row-major order.
pub fn new_matrix3(
    // Row 0
    r0c0: f32, r0c1: f32, r0c2: f32,
    // Row 1
    r1c0: f32, r1c1: f32, r1c2: f32,
    // Row 2
    r2c0: f32, r2c1: f32, r2c2: f32,
) -> Matrix {
    new_matrix4(
        r0c0, r0c1, r0c2, 0.0,
        r1c0, r1c1, r1c2, 0.0,
        r2c0, r2c1, r2c2, 0.0,
        0.0,  0.0,  0.0,  1.0,
    )
}

/// Creates a model matrix combining translation, scale, and rotation
pub fn create_model_matrix(translation: Vector3, scale: f32, rotation: Vector3) -> Matrix {
    let (sx, sy, sz) = (scale, scale, scale); // uniform scaling
    // Scale
    let ms = Matrix {
        m0: sx, m4: 0.0, m8: 0.0,  m12: 0.0,
        m1: 0.0, m5: sy, m9: 0.0,  m13: 0.0,
        m2: 0.0, m6: 0.0, m10: sz, m14: 0.0,
        m3: 0.0, m7: 0.0, m11: 0.0, m15: 1.0,
    };
    // Rotation X
    let (sxn, cxn) = rotation.x.sin_cos();
    let rx = Matrix {
        m0: 1.0, m4: 0.0,  m8: 0.0,   m12: 0.0,
        m1: 0.0, m5: cxn,  m9: -sxn,  m13: 0.0,
        m2: 0.0, m6: sxn,  m10: cxn,  m14: 0.0,
        m3: 0.0, m7: 0.0,  m11: 0.0,  m15: 1.0,
    };
    // Rotation Y
    let (syn, cyn) = rotation.y.sin_cos();
    let ry = Matrix {
        m0: cyn,  m4: 0.0, m8: syn,  m12: 0.0,
        m1: 0.0,  m5: 1.0, m9: 0.0,  m13: 0.0,
        m2: -syn, m6: 0.0, m10: cyn, m14: 0.0,
        m3: 0.0,  m7: 0.0, m11: 0.0, m15: 1.0,
    };
    // Rotation Z
    let (szn, czn) = rotation.z.sin_cos();
    let rz = Matrix {
        m0: czn,  m4: -szn, m8: 0.0, m12: 0.0,
        m1: szn,  m5:  czn, m9: 0.0, m13: 0.0,
        m2: 0.0,  m6:  0.0, m10: 1.0, m14: 0.0,
        m3: 0.0,  m7:  0.0, m11: 0.0, m15: 1.0,
    };
    // Translation
    let mt = Matrix {
        m0: 1.0, m4: 0.0, m8: 0.0, m12: translation.x,
        m1: 0.0, m5: 1.0, m9: 0.0, m13: translation.y,
        m2: 0.0, m6: 0.0, m10: 1.0, m14: translation.z,
        m3: 0.0, m7: 0.0, m11: 0.0, m15: 1.0,
    };

    // Compose: M = T · Rz · Ry · Rx · S
    let mrs = multiply_matrix_matrix(&rz, &ry);
    let mrs = multiply_matrix_matrix(&mrs, &rx);
    let mrs = multiply_matrix_matrix(&mrs, &ms);
    let mrst = multiply_matrix_matrix(&mt, &mrs);
    mrst
}
/// Creates a view matrix using camera position, target, and up vector
/// This implements a lookAt matrix for camera transformations
pub fn create_view_matrix(eye: Vector3, target: Vector3, up: Vector3) -> Matrix {
    // Calculate forward vector (from eye to target, normalized)
    let mut forward = Vector3::new(
        target.x - eye.x,
        target.y - eye.y,
        target.z - eye.z,
    );
    // Normalize forward
    let forward_length = (forward.x * forward.x + forward.y * forward.y + forward.z * forward.z).sqrt();
    forward.x /= forward_length;
    forward.y /= forward_length;
    forward.z /= forward_length;

    // Calculate right vector (cross product of forward and up, normalized)
    let mut right = Vector3::new(
        forward.y * up.z - forward.z * up.y,
        forward.z * up.x - forward.x * up.z,
        forward.x * up.y - forward.y * up.x,
    );
    // Normalize right
    let right_length = (right.x * right.x + right.y * right.y + right.z * right.z).sqrt();
    right.x /= right_length;
    right.y /= right_length;
    right.z /= right_length;

    // Calculate actual up vector (cross product of right and forward)
    let actual_up = Vector3::new(
        right.y * forward.z - right.z * forward.y,
        right.z * forward.x - right.x * forward.z,
        right.x * forward.y - right.y * forward.x,
    );

    // Create the view matrix (inverse of camera transformation)
    // This is the lookAt matrix formula
    new_matrix4(
        right.x, right.y, right.z, -(right.x * eye.x + right.y * eye.y + right.z * eye.z),
        actual_up.x, actual_up.y, actual_up.z, -(actual_up.x * eye.x + actual_up.y * eye.y + actual_up.z * eye.z),
        -forward.x, -forward.y, -forward.z, forward.x * eye.x + forward.y * eye.y + forward.z * eye.z,
        0.0, 0.0, 0.0, 1.0,
    )
}

/// Creates a perspective projection matrix
/// fov_y: Field of view in radians (vertical)
/// aspect: Aspect ratio (width / height)
/// near: Near clipping plane distance
/// far: Far clipping plane distance
pub fn create_projection_matrix(fov_y: f32, aspect: f32, near: f32, far: f32) -> Matrix {
    let tan_half_fov = (fov_y / 2.0).tan();

    new_matrix4(
        1.0 / (aspect * tan_half_fov), 0.0, 0.0, 0.0,
        0.0, 1.0 / tan_half_fov, 0.0, 0.0,
        0.0, 0.0, -(far + near) / (far - near), -(2.0 * far * near) / (far - near),
        0.0, 0.0, -1.0, 0.0,
    )
}

/// Creates a viewport matrix to transform NDC coordinates to screen space
/// x, y: Viewport position (typically 0, 0)
/// width, height: Viewport dimensions in pixels
pub fn create_viewport_matrix(x: f32, y: f32, width: f32, height: f32) -> Matrix {
    let half_width = width / 2.0;
    let half_height = height / 2.0;

    new_matrix4(
        half_width, 0.0, 0.0, x + half_width,
        0.0, -half_height, 0.0, y + half_height,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    )
}
pub fn multiply_matrix_matrix(a: &Matrix, b: &Matrix) -> Matrix {
    Matrix {
        m0:  a.m0*b.m0  + a.m4*b.m1  + a.m8*b.m2  + a.m12*b.m3,
        m1:  a.m1*b.m0  + a.m5*b.m1  + a.m9*b.m2  + a.m13*b.m3,
        m2:  a.m2*b.m0  + a.m6*b.m1  + a.m10*b.m2 + a.m14*b.m3,
        m3:  a.m3*b.m0  + a.m7*b.m1  + a.m11*b.m2 + a.m15*b.m3,

        m4:  a.m0*b.m4  + a.m4*b.m5  + a.m8*b.m6  + a.m12*b.m7,
        m5:  a.m1*b.m4  + a.m5*b.m5  + a.m9*b.m6  + a.m13*b.m7,
        m6:  a.m2*b.m4  + a.m6*b.m5  + a.m10*b.m6 + a.m14*b.m7,
        m7:  a.m3*b.m4  + a.m7*b.m5  + a.m11*b.m6 + a.m15*b.m7,

        m8:  a.m0*b.m8  + a.m4*b.m9  + a.m8*b.m10 + a.m12*b.m11,
        m9:  a.m1*b.m8  + a.m5*b.m9  + a.m9*b.m10 + a.m13*b.m11,
        m10: a.m2*b.m8  + a.m6*b.m9  + a.m10*b.m10+ a.m14*b.m11,
        m11: a.m3*b.m8  + a.m7*b.m9  + a.m11*b.m10+ a.m15*b.m11,

        m12: a.m0*b.m12 + a.m4*b.m13 + a.m8*b.m14 + a.m12*b.m15,
        m13: a.m1*b.m12 + a.m5*b.m13 + a.m9*b.m14 + a.m13*b.m15,
        m14: a.m2*b.m12 + a.m6*b.m13 + a.m10*b.m14+ a.m14*b.m15,
        m15: a.m3*b.m12 + a.m7*b.m13 + a.m11*b.m14+ a.m15*b.m15,
    }
}