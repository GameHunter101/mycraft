use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use gamezap::model::{Mesh, MeshManager, MeshTransform, Vertex};
use nalgebra as na;
use threadpool::ThreadPool;
use wgpu::util::DeviceExt;

use crate::{
    cube::{BlockWrapper, Blocks, Cube},
    utils::MeshTools,
};

pub const X_SIZE: usize = 16;
pub const Y_SIZE: usize = 256;
pub const Z_SIZE: usize = 16;

pub const MAX_VERTICES: usize = 24 * Z_SIZE * X_SIZE * Y_SIZE / 2;
pub const MAX_INDICES: usize = 36 * Z_SIZE * X_SIZE * Y_SIZE / 2;

pub type BlockArray = [[[u16; Z_SIZE]; X_SIZE]; Y_SIZE];

#[derive(Debug)]
pub struct Chunk {
    pub position: na::Vector2<i32>,
    pub blocks: &'static BlockArray,
    pub atlas_material_index: u32,
}

impl Chunk {
    fn query_block(&self, x: usize, y: usize, z: usize) -> Blocks {
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

    pub const fn default_blocks() -> BlockArray {
        let mut blocks: BlockArray = [[[1; Z_SIZE]; X_SIZE]; Y_SIZE];
        blocks[Y_SIZE - 1] = [[0; Z_SIZE]; X_SIZE];
        blocks
    }
}

impl MeshTools for Chunk {
    fn create_mesh(&self, device: &wgpu::Device, mesh_manager: Arc<Mutex<MeshManager>>) {
        let mut vertices = vec![Vertex::blank(); MAX_VERTICES].into_boxed_slice();
        let mut total_vertex_count = 0_usize;

        let mut indices = vec![0_u32; MAX_INDICES].into_boxed_slice();
        let mut total_index_count = 0_usize;

        let beginning = Instant::now();
        for (y, slice) in self.blocks.iter().enumerate() {
            for (x, row) in slice.iter().enumerate() {
                for (z, block) in row.iter().enumerate() {
                    let block_type = BlockWrapper[*block as u32];

                    if block_type != Blocks::Null {
                        let face_mask = self.query_neighbors(x, y, z);
                        if face_mask != 0 {
                            let start_first_timer = Instant::now();
                            let block = Cube::new(
                                "cube",
                                na::Vector3::new(
                                    self.position.x as f32,
                                    0.0,
                                    self.position.y as f32,
                                ) * 16.0
                                    + na::Vector3::new(x as f32, y as f32, z as f32),
                                0,
                                block_type,
                                face_mask,
                                true,
                            );
                            let first_timer = Instant::now() - start_first_timer;
                            if first_timer.as_micros() > 0 {
                                // println!("Second timer: {:?}", first_timer);
                            }

                            block.mesh_info.indices.iter().for_each(|&i| {
                                if i != u32::MAX {
                                    indices[total_index_count] = i + total_vertex_count as u32;
                                    total_index_count += 1;
                                }
                            });

                            let start_second_timer = Instant::now();
                            let block_vertex_count = block.mesh_info.vertices.len();
                            let vert_slice = &mut vertices[total_vertex_count..total_vertex_count + block_vertex_count];
                            for (i, vert_slot) in vert_slice.iter_mut().enumerate() {
                                *vert_slot = block.mesh_info.vertices[i];
                            }
                            total_vertex_count += block_vertex_count;
                            let second_timer = Instant::now() - start_second_timer;
                            if second_timer.as_micros() > 0 {
                                // println!("Second timer: {:?}", second_timer);
                            }
                        }
                    }
                }
            }
        }
        let elapsed = Instant::now() - beginning;
        if elapsed.as_micros() > 0 {
            // println!("Time elapsed: {:?}", elapsed);
        }

        let vertices = &vertices[..total_vertex_count];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk 1 vert buff"),
            usage: wgpu::BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(vertices),
        });

        let indices = &indices[..total_index_count];
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk 1 index buff"),
            usage: wgpu::BufferUsages::INDEX,
            contents: bytemuck::cast_slice(indices),
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
        mesh_manager
            .lock()
            .unwrap()
            .diffuse_pipeline_models
            .push(mesh);
    }
}
