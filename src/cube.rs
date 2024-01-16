use std::sync::{Mutex, MutexGuard};

use gamezap::model::Vertex;
use lazy_static::lazy_static;
use nalgebra as na;

use crate::ATLAS_SIZE;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Blocks {
    Grass,
    Dirt,
    Null,
}

const FACE_TEXTURE_OFFSET: f32 = 16.0 / ATLAS_SIZE;
pub const FACE_INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];

#[derive(Debug)]
pub struct BlockWrapper;

impl std::ops::Index<u16> for BlockWrapper {
    type Output = Blocks;

    fn index(&self, index: u16) -> &Self::Output {
        match index {
            0 => &Blocks::Grass,
            1 => &Blocks::Dirt,
            _ => &Blocks::Null,
        }
    }
}

impl Blocks {
    fn coords(&self) -> (f32, f32) {
        match self {
            Blocks::Grass => (0.0, 0.0),
            Blocks::Dirt => (0.0, FACE_TEXTURE_OFFSET),
            Blocks::Null => (1.0, 1.0),
        }
    }
}

pub type Face = [Vertex; 4];

pub struct MeshInfo {
    pub faces: [Face; 6],
    pub face_count: usize,
}

impl MeshInfo {
    pub fn init() -> Self {
        MeshInfo {
            faces: [[Vertex::blank(); 4]; 6],
            face_count: 0,
        }
    }

    pub fn full(pos: na::Vector3<f32>) -> Self {
        let mut mesh = MeshInfo::init();
        mesh.append_data(NEGATIVE_X_FACE.lock().unwrap(), (0.0, 0.0), pos);
        mesh.append_data(NEGATIVE_Y_FACE.lock().unwrap(), (0.0, 0.0), pos);
        mesh.append_data(NEGATIVE_Z_FACE.lock().unwrap(), (0.0, 0.0), pos);
        mesh.append_data(POSITIVE_X_FACE.lock().unwrap(), (0.0, 0.0), pos);
        mesh.append_data(POSITIVE_Y_FACE.lock().unwrap(), (0.0, 0.0), pos);
        mesh.append_data(POSITIVE_Z_FACE.lock().unwrap(), (0.0, 0.0), pos);
        mesh
    }

    pub fn append_data(
        &mut self,
        new_vertices: MutexGuard<Face>,
        coords: (f32, f32),
        vertex_offset: na::Vector3<f32>,
    ) {
        for (i, vert) in new_vertices.iter().enumerate() {
            self.faces[self.face_count][i] = vert.translate(vertex_offset);
        }
        self.face_count += 1;
    }
}

lazy_static! {
    static ref NEGATIVE_Z_FACE: Mutex<Face> = Mutex::new([
        Vertex {
            position: [0.0, 1.0, 0.0],
            tex_coords: [FACE_TEXTURE_OFFSET, 0.0],
            normal: [0.0, 0.0, -1.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [0.0, 0.0, 0.0],
            tex_coords: [FACE_TEXTURE_OFFSET, FACE_TEXTURE_OFFSET],
            normal: [0.0, 0.0, -1.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [1.0, 0.0, 0.0],
            tex_coords: [2.0 * FACE_TEXTURE_OFFSET, FACE_TEXTURE_OFFSET],
            normal: [0.0, 0.0, -1.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [1.0, 1.0, 0.0],
            tex_coords: [2.0 * FACE_TEXTURE_OFFSET, 0.0],
            normal: [0.0, 0.0, -1.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        }
    ]);
    static ref POSITIVE_Z_FACE: Mutex<Face> = Mutex::new([
        Vertex {
            position: [0.0, 1.0, 1.0],
            tex_coords: [FACE_TEXTURE_OFFSET, 0.0],
            normal: [0.0, 0.0, 1.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [0.0, 0.0, 1.0],
            tex_coords: [FACE_TEXTURE_OFFSET, FACE_TEXTURE_OFFSET],
            normal: [0.0, 0.0, 1.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [1.0, 0.0, 1.0],
            tex_coords: [2.0 * FACE_TEXTURE_OFFSET, FACE_TEXTURE_OFFSET],
            normal: [0.0, 0.0, 1.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [1.0, 1.0, 1.0],
            tex_coords: [2.0 * FACE_TEXTURE_OFFSET, 0.0],
            normal: [0.0, 0.0, 1.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
    ]);
    static ref NEGATIVE_X_FACE: Mutex<Face> = Mutex::new([
        Vertex {
            position: [0.0, 1.0, 0.0],
            tex_coords: [FACE_TEXTURE_OFFSET, 0.0],
            normal: [-1.0, 0.0, 0.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [0.0, 0.0, 0.0],
            tex_coords: [FACE_TEXTURE_OFFSET, FACE_TEXTURE_OFFSET],
            normal: [-1.0, 0.0, 0.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [0.0, 0.0, 1.0],
            tex_coords: [2.0 * FACE_TEXTURE_OFFSET, FACE_TEXTURE_OFFSET],
            normal: [-1.0, 0.0, 0.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [0.0, 1.0, 1.0],
            tex_coords: [2.0 * FACE_TEXTURE_OFFSET, 0.0],
            normal: [-1.0, 0.0, 0.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
    ]);
    static ref POSITIVE_X_FACE: Mutex<Face> = Mutex::new([
        Vertex {
            position: [1.0, 1.0, 0.0],
            tex_coords: [FACE_TEXTURE_OFFSET, 0.0],
            normal: [1.0, 0.0, 0.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [1.0, 0.0, 0.0],
            tex_coords: [FACE_TEXTURE_OFFSET, FACE_TEXTURE_OFFSET],
            normal: [1.0, 0.0, 0.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [1.0, 0.0, 1.0],
            tex_coords: [2.0 * FACE_TEXTURE_OFFSET, FACE_TEXTURE_OFFSET],
            normal: [1.0, 0.0, 0.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [1.0, 1.0, 1.0],
            tex_coords: [2.0 * FACE_TEXTURE_OFFSET, 0.0],
            normal: [1.0, 0.0, 0.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
    ]);
    static ref NEGATIVE_Y_FACE: Mutex<Face> = Mutex::new([
        Vertex {
            position: [0.0, 0.0, 0.0],
            tex_coords: [2.0 * FACE_TEXTURE_OFFSET, 0.0],
            normal: [0.0, -1.0, 0.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [0.0, 0.0, 1.0],
            tex_coords: [2.0 * FACE_TEXTURE_OFFSET, FACE_TEXTURE_OFFSET],
            normal: [0.0, -1.0, 0.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [1.0, 0.0, 1.0],
            tex_coords: [3.0 * FACE_TEXTURE_OFFSET, FACE_TEXTURE_OFFSET],
            normal: [0.0, -1.0, 0.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [1.0, 0.0, 0.0],
            tex_coords: [3.0 * FACE_TEXTURE_OFFSET, 0.0],
            normal: [0.0, -1.0, 0.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
    ]);
    static ref POSITIVE_Y_FACE: Mutex<Face> = Mutex::new([
        Vertex {
            position: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 1.0, 0.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [0.0, 1.0, 1.0],
            tex_coords: [0.0, FACE_TEXTURE_OFFSET],
            normal: [0.0, 1.0, 0.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [1.0, 1.0, 1.0],
            tex_coords: [FACE_TEXTURE_OFFSET, FACE_TEXTURE_OFFSET],
            normal: [0.0, 1.0, 0.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
        Vertex {
            position: [1.0, 1.0, 0.0],
            tex_coords: [FACE_TEXTURE_OFFSET, 0.0],
            normal: [0.0, 1.0, 0.0],
            bitangent: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
        },
    ]);
}

pub struct Cube {
    pub position: na::Vector3<f32>,
    pub material_index: u32,
    pub mesh_info: MeshInfo,
}

impl Cube {
    pub fn new(
        position: na::Vector3<f32>,
        material_index: u32,
        block: Blocks,
        face_mask: u8,
        set_verts_to_positon: bool,
    ) -> Self {
        let texture_coords = block.coords();

        let mut mesh_info = MeshInfo::init();

        let vertex_offset = if set_verts_to_positon {
            position
        } else {
            na::Vector3::new(0.0, 0.0, 0.0)
        };

        // Front face
        if face_mask & 0b0000_0001 == 0b0000_0001 {
            mesh_info.append_data(
                NEGATIVE_Z_FACE.lock().unwrap(),
                // &face_indices,
                texture_coords,
                vertex_offset,
            );
        }
        // Back face
        if face_mask & 0b0000_0010 == 0b0000_0010 {
            mesh_info.append_data(
                POSITIVE_Z_FACE.lock().unwrap(),
                // &face_indices,
                texture_coords,
                vertex_offset,
            );
        }
        // Left face
        if face_mask & 0b0000_0100 == 0b0000_0100 {
            mesh_info.append_data(
                NEGATIVE_X_FACE.lock().unwrap(),
                // &face_indices,
                texture_coords,
                vertex_offset,
            );
        }
        // Right face
        if face_mask & 0b0000_1000 == 0b0000_1000 {
            mesh_info.append_data(
                POSITIVE_X_FACE.lock().unwrap(),
                // &face_indices,
                texture_coords,
                vertex_offset,
            );
        }
        // Bottom face
        if face_mask & 0b0001_0000 == 0b0001_0000 {
            mesh_info.append_data(
                NEGATIVE_Y_FACE.lock().unwrap(),
                // &face_indices,
                texture_coords,
                vertex_offset,
            );
        }
        // Top face
        if face_mask & 0b0010_0000 == 0b0010_0000 {
            mesh_info.append_data(
                POSITIVE_Y_FACE.lock().unwrap(),
                // &face_indices,
                texture_coords,
                vertex_offset,
            );
        }

        Cube {
            position,
            material_index,
            mesh_info,
        }
    }
}
