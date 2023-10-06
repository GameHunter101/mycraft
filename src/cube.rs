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

pub struct BlockWrapper;

impl std::ops::Index<u32> for BlockWrapper {
    type Output = Blocks;

    fn index(&self, index: u32) -> &Self::Output {
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

pub struct MeshInfo {
    pub vertices: [Vertex; 24],
    // pub indices: [u32; 36],
    pub vertex_count: usize,
    // index_count: usize,
    // previous_index: u32,
}

impl MeshInfo {
    pub fn init() -> Self {
        MeshInfo {
            vertices: [Vertex::blank(); 24],
            // indices: [u32::MAX; 36],
            vertex_count: 0,
            // index_count: 0,
            // previous_index: 0,
        }
    }

    pub fn append_data(
        &mut self,
        new_vertices: MutexGuard<[Vertex; 4]>,
        // new_indices: &[u32; 6],
        coords: (f32, f32),
        vertex_offset: na::Vector3<f32>,
    ) {
        let slice = &mut self.vertices[self.vertex_count..self.vertex_count + 4];
        for (i, old_vert) in slice.iter_mut().enumerate() {
            let old_position = na::Vector3::new(
                new_vertices[i].position[0],
                new_vertices[i].position[1],
                new_vertices[i].position[2],
            );
            *old_vert = new_vertices[i];
            old_vert.position = (old_position + vertex_offset).into();
            old_vert.tex_coords[0] += coords.0;
            old_vert.tex_coords[1] += coords.1;
        }
        self.vertex_count += 4;

        // let slice = &mut self.indices[self.index_count..self.index_count + 6];
        // for (i, old_index) in slice.iter_mut().enumerate() {
        //     *old_index = new_indices[i] + self.previous_index;
        // }
        // self.previous_index += new_indices[5] + 1;
        // self.index_count += 6;
    }
}

lazy_static! {
    static ref NEGATIVE_Z_FACE: Mutex<[Vertex; 4]> = Mutex::new([
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
        },
    ]);
    static ref POSITIVE_Z_FACE: Mutex<[Vertex; 4]> = Mutex::new([
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
    static ref NEGATIVE_X_FACE: Mutex<[Vertex; 4]> = Mutex::new([
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
    static ref POSITIVE_X_FACE: Mutex<[Vertex; 4]> = Mutex::new([
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
    static ref NEGATIVE_Y_FACE: Mutex<[Vertex; 4]> = Mutex::new([
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
    static ref POSITIVE_Y_FACE: Mutex<[Vertex; 4]> = Mutex::new([
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
