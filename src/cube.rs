use std::cell::RefMut;

use gamezap::model::{Mesh, MeshManager, MeshTransform, Vertex};
use nalgebra as na;
use wgpu::util::DeviceExt;

use crate::{utils::MeshTools, ATLAS_SIZE};

#[derive(Debug, Clone, Copy)]
pub enum Blocks {
    Grass,
    Dirt,
    Null,
}

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
            Blocks::Dirt => (0.0, 16.0 / ATLAS_SIZE),
            Blocks::Null => (1.0, 1.0),
        }
    }
}

pub struct Cube {
    pub name: String,
    pub position: na::Vector3<f32>,
    pub material_index: u32,
    pub vertices: [Vertex; 24],
    pub indices: [u32; 36],
}

impl Cube {
    pub fn new(name: &str, position: na::Vector3<f32>, material_index: u32, block: Blocks) -> Self {
        let texture_coords = block.coords();
        let face_size = 16.0 / ATLAS_SIZE;

        let top_face = (texture_coords.0, texture_coords.1);
        let side_face = (texture_coords.0 + face_size, texture_coords.1);
        let bottom_face = (texture_coords.0 + 2.0 * face_size, texture_coords.1);

        let vertices = [
            // Front face
            Vertex {
                position: [0.0, 1.0, 0.0],
                tex_coords: [side_face.0, side_face.1],
                normal: [0.0, 0.0, -1.0],
                bitangent: [0.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.0, 0.0, 0.0],
                tex_coords: [side_face.0, side_face.1 + face_size],
                normal: [0.0, 0.0, -1.0],
                bitangent: [0.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [1.0, 0.0, 0.0],
                tex_coords: [side_face.0 + face_size, side_face.1 + face_size],
                normal: [0.0, 0.0, -1.0],
                bitangent: [0.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [1.0, 1.0, 0.0],
                tex_coords: [side_face.0 + face_size, side_face.1],
                normal: [0.0, 0.0, -1.0],
                bitangent: [0.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
            },
            // Right face
            Vertex {
                position: [1.0, 1.0, 0.0],
                tex_coords: [side_face.0, side_face.1],
                normal: [1.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [1.0, 0.0, 0.0],
                tex_coords: [side_face.0, side_face.1 + face_size],
                normal: [1.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [1.0, 0.0, 1.0],
                tex_coords: [side_face.0 + face_size, side_face.1 + face_size],
                normal: [1.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [1.0, 1.0, 1.0],
                tex_coords: [side_face.0 + face_size, side_face.1],
                normal: [1.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
            },
            // Back face
            Vertex {
                position: [0.0, 1.0, 1.0],
                tex_coords: [side_face.0, side_face.1],
                normal: [0.0, 0.0, 1.0],
                bitangent: [0.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.0, 0.0, 1.0],
                tex_coords: [side_face.0, side_face.1 + face_size],
                normal: [0.0, 0.0, 1.0],
                bitangent: [0.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [1.0, 0.0, 1.0],
                tex_coords: [side_face.0 + face_size, side_face.1 + face_size],
                normal: [0.0, 0.0, 1.0],
                bitangent: [0.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [1.0, 1.0, 1.0],
                tex_coords: [side_face.0 + face_size, side_face.1],
                normal: [0.0, 0.0, 1.0],
                bitangent: [0.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
            },
            // Left face
            Vertex {
                position: [0.0, 1.0, 0.0],
                tex_coords: [side_face.0, side_face.1],
                normal: [-1.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.0, 0.0, 0.0],
                tex_coords: [side_face.0, side_face.1 + face_size],
                normal: [-1.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.0, 0.0, 1.0],
                tex_coords: [side_face.0 + face_size, side_face.1 + face_size],
                normal: [-1.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.0, 1.0, 1.0],
                tex_coords: [side_face.0 + face_size, side_face.1],
                normal: [-1.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
            },
            // Top face
            Vertex {
                position: [1.0, 1.0, 0.0],
                tex_coords: [top_face.0, top_face.1],
                normal: [0.0, 1.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [1.0, 1.0, 1.0],
                tex_coords: [top_face.0, top_face.1 + face_size],
                normal: [0.0, 1.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.0, 1.0, 1.0],
                tex_coords: [top_face.0 + face_size, top_face.1 + face_size],
                normal: [0.0, 1.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.0, 1.0, 0.0],
                tex_coords: [top_face.0 + face_size, top_face.1],
                normal: [0.0, 1.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
            },
            // Bottom face
            Vertex {
                position: [1.0, 0.0, 0.0],
                tex_coords: [bottom_face.0, bottom_face.1],
                normal: [0.0, -1.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [1.0, 0.0, 1.0],
                tex_coords: [bottom_face.0, bottom_face.1 + face_size],
                normal: [0.0, -1.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.0, 0.0, 1.0],
                tex_coords: [bottom_face.0 + face_size, bottom_face.1 + face_size],
                normal: [0.0, -1.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.0, 0.0, 0.0],
                tex_coords: [bottom_face.0 + face_size, top_face.1],
                normal: [0.0, -1.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
            },
        ];
        let indices: [u32; 36] = [
            0, 1, 2, 0, 2, 3, // Front face
            4, 5, 6, 4, 6, 7, // Right face
            8, 9, 10, 8, 10, 11, // Back face
            12, 13, 14, 12, 14, 15, //Left face
            16, 17, 18, 16, 18, 19, // Top face
            20, 21, 22, 20, 22, 23, // Bottom face
        ];
        Cube {
            name: name.to_string(),
            position,
            material_index,
            vertices,
            indices,
        }
    }
}

impl MeshTools for Cube {
    fn create_mesh(&self, device: &wgpu::Device, mut mesh_manager: RefMut<MeshManager>) {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{} Vertex Buffer", self.name)),
            usage: wgpu::BufferUsages::VERTEX,
            contents: &bytemuck::cast_slice(&self.vertices),
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{} Index Buffer", self.name)),
            usage: wgpu::BufferUsages::INDEX,
            contents: &bytemuck::cast_slice(&self.indices),
        });

        let mesh = Mesh::new(
            device,
            self.name.clone(),
            vertex_buffer,
            index_buffer,
            self.indices.len() as u32,
            MeshTransform::new(
                self.position,
                na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), 0.0),
            ),
            0,
        );
        mesh_manager.diffuse_pipeline_models.push(mesh);
    }
}
