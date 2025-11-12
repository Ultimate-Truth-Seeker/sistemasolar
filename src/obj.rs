use raylib::math::{Vector2, Vector3};
use tobj;

pub struct Obj {
    pub vertices: Vec<Vector3>,
    pub indices: Vec<u32>,
}

impl Obj {
    pub fn load(path: &str) -> Result<Self, tobj::LoadError> {
        let (models, _materials) = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS)?;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for model in models {
            let mesh = &model.mesh;
            let num_vertices = mesh.positions.len() / 3;

            for i in 0..num_vertices {
                let x = mesh.positions[i * 3];
                let y = mesh.positions[i * 3 + 1];
                let z = mesh.positions[i * 3 + 2];
                let position = Vector3::new(x, y, z);
                vertices.push(position);
            }
            indices.extend_from_slice(&mesh.indices);
        }

        Ok(Obj { vertices, indices })
    }

    pub fn get_vertex_array(&self) -> Vec<Vector3> {
        let mut vertex_array = Vec::new();
        for &index in &self.indices {
            vertex_array.push(self.vertices[index as usize].clone());
        }
        vertex_array
    }
}