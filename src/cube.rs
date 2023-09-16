use std::sync::{Arc, Mutex, MutexGuard};

use gamezap::model::{Mesh, MeshManager, MeshTransform, Vertex};
use lazy_static::lazy_static;
use nalgebra as na;
use wgpu::util::DeviceExt;

use crate::{utils::MeshTools, ATLAS_SIZE};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Blocks {
    Grass,
    Dirt,
    Null,
}

const FACE_TEXTURE_OFFSET: f32 = 16.0 / ATLAS_SIZE;

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
    pub indices: Vec<u32>,
    vertex_count: usize,
    index_count: usize,
    previous_index: u32,
}

impl MeshInfo {
    pub fn init() -> Self {
        MeshInfo {
            vertices: [Vertex::blank(); 24],
            indices: vec![],
            vertex_count: 0,
            index_count: 0,
            previous_index: 0,
        }
    }

    pub fn append_data(
        &mut self,
        new_vertices: MutexGuard<[Vertex; 4]>,
        new_indices: &[u32; 6],
        coords: (f32, f32),
        vertex_offset: na::Vector3<f32>,
    ) {
        let slice = &mut self.vertices[self.vertex_count..self.vertex_count + 4];
        for (i, old_vert) in slice.iter_mut().enumerate() {
            let old_position = na::Vector3::new(new_vertices[i].position[0], new_vertices[i].position[1], new_vertices[i].position[2]);
            *old_vert = new_vertices[i];
            old_vert.position = (old_position + vertex_offset).into();
            old_vert.tex_coords[0] += coords.0;
            old_vert.tex_coords[1] += coords.1;
        }
        // self.vertices.append(&mut new_vertices.to_vec());
        self.indices.append(
            &mut new_indices
                .iter()
                .map(|&i| {
                    i + if let Some(p) = self.indices.last() {
                        p + 1
                    } else {
                        0
                    }
                })
                .collect::<Vec<_>>(),
        );
        // for (i, vert) in verts.iter().enumerate() {
        //     self.vertices[self.vertex_count + i] = *vert;
        // }
        self.vertex_count += 4;

        // for (i, index) in indices.iter().enumerate() {
        //     self.indices[self.index_count + i] = self.previous_index + *index;
        // }
        // self.index_count += indices.len();
        // self.previous_index = self.indices[self.index_count - 1];
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
    pub name: String,
    pub position: na::Vector3<f32>,
    pub material_index: u32,
    pub mesh_info: MeshInfo,
}

impl Cube {
    pub fn new(
        name: &str,
        position: na::Vector3<f32>,
        material_index: u32,
        block: Blocks,
        face_mask: u8,
        set_verts_to_positon: bool,
    ) -> Self {
        let texture_coords = block.coords();
        let face_size = 16.0 / ATLAS_SIZE;

        let top_face = (texture_coords.0, texture_coords.1);
        let side_face = (texture_coords.0, texture_coords.1);
        let bottom_face = (texture_coords.0 + 2.0 * face_size, texture_coords.1);

        let mut mesh_info = MeshInfo::init();

        let face_indices = [0, 1, 2, 0, 2, 3];

        let vertex_offset =
            if set_verts_to_positon {
                position
            } else {
                na::Vector3::new(0.0, 0.0, 0.0)
            };

        // Front face
        if face_mask & 0b0000_0001 == 0b0000_0001 {
            mesh_info.append_data(
                NEGATIVE_Z_FACE.lock().unwrap(),
                &face_indices,
                texture_coords,
                vertex_offset,
            );
        }
        // Back face
        if face_mask & 0b0000_0010 == 0b0000_0010 {
            mesh_info.append_data(
                POSITIVE_Z_FACE.lock().unwrap(),
                &face_indices,
                texture_coords,
                vertex_offset,
            );
        }
        // Left face
        if face_mask & 0b0000_0100 == 0b0000_0100 {
            mesh_info.append_data(
                NEGATIVE_X_FACE.lock().unwrap(),
                &face_indices,
                texture_coords,
                vertex_offset,
            );
        }
        // Right face
        if face_mask & 0b0000_1000 == 0b0000_1000 {
            mesh_info.append_data(
                POSITIVE_X_FACE.lock().unwrap(),
                &face_indices,
                texture_coords,
                vertex_offset,
            );
        }
        // Bottom face
        if face_mask & 0b0001_0000 == 0b0001_0000 {
            mesh_info.append_data(
                NEGATIVE_Y_FACE.lock().unwrap(),
                &face_indices,
                texture_coords,
                vertex_offset,
            );
        }
        // Top face
        if face_mask & 0b0010_0000 == 0b0010_0000 {
            mesh_info.append_data(
                POSITIVE_Y_FACE.lock().unwrap(),
                &face_indices,
                texture_coords,
                vertex_offset,
            );
        }

        // let indices: Vec<u32> = vec![
        //     0, 1, 2, 0, 2, 3, // Front face
        //     4, 5, 6, 4, 6, 7, // Right face
        //     8, 9, 10, 8, 10, 11, // Back face
        //     12, 13, 14, 12, 14, 15, //Left face
        //     16, 17, 18, 16, 18, 19, // Top face
        //     20, 21, 22, 20, 22, 23, // Bottom face
        // ];
        // println!("{:?}", mesh_info.indices);

        Cube {
            name: name.to_string(),
            position,
            material_index,
            mesh_info,
        }
    }

    fn update_mesh_info(
        vert_array: &mut [Vertex; 24],
        index_array: &mut [u32; 36],
        vert_offset: &mut usize,
        index_offset: &mut usize,
        verts: [Vertex; 4],
        indices: [u32; 6],
    ) {
        for (i, vert) in verts.iter().enumerate() {
            vert_array[*vert_offset + i] = *vert;
        }
        *vert_offset += 4;
        let index_before = match index_array[*index_offset] {
            u32::MAX => 0,
            e => e,
        };
        dbg!(index_before, &index_offset);
        for (i, ind) in indices.iter().enumerate() {
            index_array[*index_offset + i] = index_before + *ind;
        }
        *index_offset += 6;
    }
}

impl MeshTools for Cube {
    fn create_mesh(&self, device: &wgpu::Device, mesh_manager: Arc<Mutex<MeshManager>>) {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{} Vertex Buffer", self.name)),
            usage: wgpu::BufferUsages::VERTEX,
            contents: &bytemuck::cast_slice(
                &self.mesh_info.vertices[..self.mesh_info.vertex_count],
            ),
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{} Index Buffer", self.name)),
            usage: wgpu::BufferUsages::INDEX,
            contents: &bytemuck::cast_slice(&self.mesh_info.indices),
        });

        let mesh = Mesh::new(
            device,
            self.name.clone(),
            vertex_buffer,
            index_buffer,
            self.mesh_info.index_count as u32,
            MeshTransform::new(
                self.position,
                na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), 0.0),
            ),
            0,
        );
        mesh_manager
            .lock()
            .unwrap()
            .diffuse_pipeline_models
            .push(mesh);
    }
}
