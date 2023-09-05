use gamezap::model::{Mesh, MeshTransform, Vertex};
use nalgebra as na;
use wgpu::util::DeviceExt;

use crate::{
    cube::{BlockWrapper, Blocks, Cube},
    utils::MeshTools,
};

pub const X_SIZE: usize = 16;
pub const Y_SIZE: usize = 256;
pub const Z_SIZE: usize = 16;

pub struct Chunk {
    pub position: na::Vector3<f32>,
    pub blocks: Box<[[[u16; Z_SIZE]; X_SIZE]; Y_SIZE]>,
    pub atlas_material_index: u32,
}

impl Chunk {
    fn query_block(&self, x: usize, y: usize, z: usize) -> Blocks {
        // println!("{},{},{}", x, y, z);
        BlockWrapper[self.blocks[y][x][z] as u32]
    }

    /// Returns a bitmask of which faces have neighboring blocks
    ///
    /// From right to left: `1: -z, 2: +z, 4: -x, 8: +x, 16: +y, 32: -y`
    fn query_neighbors(&self, x: usize, y: usize, z: usize) -> u8 {
        let mut neighbors = 0b0000_0000;

        if z > 0 {
            if self.query_block(x, y, z - 1) == Blocks::Null {
                neighbors |= 0b0000_0001;
            }
        } else {
            neighbors |= 0b0000_0001;
        }
        if z < Z_SIZE - 1 {
            if self.query_block(x, y, z + 1) == Blocks::Null {
                neighbors |= 0b0000_0010;
            }
        } else {
            neighbors |= 0b0000_0010;
        }

        if x > 0 {
            if self.query_block(x - 1, y, z) == Blocks::Null {
                neighbors |= 0b0000_0100;
            }
        } else {
            neighbors |= 0b0000_0100;
        }
        if x < X_SIZE - 1 {
            if self.query_block(x + 1, y, z) == Blocks::Null {
                neighbors |= 0b0000_1000;
            }
        } else {
            neighbors |= 0b0000_1000;
        }

        if y > 0 {
            if self.query_block(x, y - 1, z) == Blocks::Null {
                neighbors |= 0b0001_000;
            }
        } else {
            neighbors |= 0b0001_0000;
        }
        if y < Y_SIZE - 1 {
            if self.query_block(x, y + 1, z) == Blocks::Null {
                neighbors |= 0b0010_0000;
            }
        } else {
            // println!("{},{},{}", x, y, z);
            neighbors |= 0b0010_0000;
        }
        neighbors
    }
}

impl MeshTools for Chunk {
    fn create_mesh(
        &self,
        device: &wgpu::Device,
        mut mesh_manager: std::cell::RefMut<gamezap::model::MeshManager>,
    ) {
        let mut vertices = vec![];
        let mut indices = vec![];
        for (y, slice) in self.blocks.iter().enumerate() {
            for (x, row) in slice.iter().enumerate() {
                for (z, block) in row.iter().enumerate() {
                    let block_type = BlockWrapper[*block as u32];
                    if block_type != Blocks::Null {
                        let face_mask = self.query_neighbors(x, y, z);
                        if face_mask != 0 {
                            let block = Cube::new(
                                "cube",
                                self.position + na::Vector3::new(x as f32, y as f32, z as f32),
                                0,
                                block_type,
                                face_mask,
                            );

                            let mut recalculated_vertices = block
                                .vertices
                                .iter()
                                .map(|v| {
                                    let old_position = na::Vector3::from_column_slice(&v.position);
                                    let new_position: [f32; 3] =
                                        (block.position + old_position).into();

                                    Vertex {
                                        position: new_position,
                                        tex_coords: v.tex_coords,
                                        normal: v.normal,
                                        bitangent: v.bitangent,
                                        tangent: v.tangent,
                                    }
                                })
                                .collect::<Vec<_>>();
                            let mut recalculated_indices = block
                                .indices
                                .iter()
                                .map(|i| i + vertices.len() as u32)
                                .collect::<Vec<_>>();
                            vertices.append(&mut recalculated_vertices);
                            indices.append(&mut recalculated_indices);
                        }
                    }
                }
            }
        }
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk 1 vert buff"),
            usage: wgpu::BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(&vertices),
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk 1 index buff"),
            usage: wgpu::BufferUsages::INDEX,
            contents: bytemuck::cast_slice(&indices),
        });

        let mesh = Mesh::new(
            device,
            "Chunk 1".to_string(),
            vertex_buffer,
            index_buffer,
            indices.len() as u32,
            MeshTransform::new(
                na::Vector3::new(0.0, 0.0, 0.0),
                na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), 0.0),
            ),
            0,
        );
        mesh_manager.diffuse_pipeline_models.push(mesh);
    }
}
