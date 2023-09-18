use std::{
    sync::{Arc, Barrier, Mutex},
    time::Instant,
};

use gamezap::model::{Mesh, MeshManager, MeshTransform, Vertex};
use lazy_static::lazy_static;
use nalgebra as na;
use threadpool::ThreadPool;
use wgpu::util::DeviceExt;

use crate::{
    chunk_loader::ALL_CHUNKS,
    cube::{BlockWrapper, Blocks, Cube, FACE_INDICES},
    utils::MeshTools,
};

pub const X_SIZE: usize = 16;
pub const Y_SIZE: usize = 256;
pub const Z_SIZE: usize = 16;

pub const MAX_VERTICES: usize = (24 * Z_SIZE * X_SIZE * Y_SIZE) / 2;
pub const MAX_INDICES: usize = (36 * Z_SIZE * X_SIZE * Y_SIZE) / 2;

lazy_static! {
    static ref ALL_INDICES: Box<[u32]> = (0..MAX_INDICES / 6)
        .flat_map(|face_index| FACE_INDICES.map(|i| i + (4 * face_index as u32)))
        .collect::<Vec<_>>()
        .into_boxed_slice();
}

pub type BlockArray = [[[u16; Z_SIZE]; X_SIZE]; Y_SIZE];

#[derive(Debug)]
pub struct Chunk {
    pub position: na::Vector2<i32>,
    pub chunk_index: usize,
    pub atlas_material_index: u32,
}

impl Chunk {
    fn query_block(chunk_index: usize, x: usize, y: usize, z: usize) -> Blocks {
        BlockWrapper[ALL_CHUNKS[chunk_index][y][x][z] as u32]
    }

    /// Returns a bitmask of which faces have neighboring blocks
    ///
    /// From right to left: `1: -z, 2: +z, 4: -x, 8: +x, 16: +y, 32: -y`
    fn query_neighbors(chunk_index: usize, x: usize, y: usize, z: usize) -> u8 {
        let mut neighbors = 0b0000_0000;

        if z > 0 {
            if Self::query_block(chunk_index, x, y, z - 1) == Blocks::Null {
                neighbors |= 0b0000_0001;
            }
        } else {
            neighbors |= 0b0000_0001;
        }
        if z < Z_SIZE - 1 {
            if Self::query_block(chunk_index, x, y, z + 1) == Blocks::Null {
                neighbors |= 0b0000_0010;
            }
        } else {
            neighbors |= 0b0000_0010;
        }

        if x > 0 {
            if Self::query_block(chunk_index, x - 1, y, z) == Blocks::Null {
                neighbors |= 0b0000_0100;
            }
        } else {
            neighbors |= 0b0000_0100;
        }
        if x < X_SIZE - 1 {
            if Self::query_block(chunk_index, x + 1, y, z) == Blocks::Null {
                neighbors |= 0b0000_1000;
            }
        } else {
            neighbors |= 0b0000_1000;
        }

        if y > 0 {
            if Self::query_block(chunk_index, x, y - 1, z) == Blocks::Null {
                neighbors |= 0b0001_000;
            }
        } else {
            neighbors |= 0b0001_0000;
        }
        if y < Y_SIZE - 1 {
            if Self::query_block(chunk_index, x, y + 1, z) == Blocks::Null {
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
        let vertices = Arc::new(Mutex::new(VertexArray::default()));

        let beginning = Instant::now();
        let chunk_ref: &'static BlockArray = &ALL_CHUNKS[self.chunk_index];
        let chunk_pos = self.position;

        let chunk_index = self.chunk_index;

        let vert_gen_barrier = Arc::new(Barrier::new(5));

        let pools = ThreadPool::new(4);
        let slices_per_thread = 256 / 4;
        let mut test_num = Arc::new(Mutex::new(0));
        for i in 0..4 {
            let vertices_clone = vertices.clone();
            let vert_gen_barrier_clone = vert_gen_barrier.clone();

            let test_clone = test_num.clone();

            pools.execute(move || {
                let y_offset = slices_per_thread * i;
                for (y, slice) in (&chunk_ref[y_offset..slices_per_thread * (i + 1)])
                    .iter()
                    .enumerate()
                {
                    for (x, row) in slice.iter().enumerate() {
                        for (z, block) in row.iter().enumerate() {
                            let block_type = BlockWrapper[*block as u32];

                            if block_type != Blocks::Null {
                                let face_mask =
                                    Self::query_neighbors(chunk_index, x, y + y_offset, z);
                                if face_mask != 0 {
                                    let block = Cube::new(
                                        na::Vector3::new(
                                            chunk_pos.x as f32,
                                            0.0,
                                            chunk_pos.y as f32,
                                        ) * 16.0
                                            + na::Vector3::new(
                                                x as f32,
                                                (y + y_offset) as f32,
                                                z as f32,
                                            ),
                                        0,
                                        block_type,
                                        face_mask,
                                        true,
                                    );

                                    // *test_clone.lock().unwrap() += 1;

                                    vertices_clone.lock().unwrap().push(
                                        &block.mesh_info.vertices[..block.mesh_info.vertex_count],
                                    );
                                }
                            }
                        }
                    }
                }
                vert_gen_barrier_clone.wait();
            });
        }
        vert_gen_barrier.wait();

        let elapsed = Instant::now() - beginning;
        if elapsed.as_micros() > 0 {
            // println!("Time elapsed: {:?}", elapsed);
        }

        let vertices_clone = vertices.clone();
        let vertex_count = vertices_clone.lock().unwrap().vertex_count;
        let vertices = &vertices_clone.lock().unwrap().vertices[..vertex_count];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk 1 vert buff"),
            usage: wgpu::BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(vertices),
        });

        let total_index_count = (vertex_count as f32 * 1.5) as usize;
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk 1 index buff"),
            usage: wgpu::BufferUsages::INDEX,
            contents: bytemuck::cast_slice(&ALL_INDICES[..total_index_count]),
        });

        let mesh = Mesh::new(
            device,
            "Chunk 1".to_string(),
            vertex_buffer,
            index_buffer,
            total_index_count as u32,
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

pub struct VertexArray {
    pub vertices: Box<[Vertex]>,
    pub vertex_count: usize,
}

impl VertexArray {
    pub fn push(&mut self, verts: &[Vertex]) {
        let start_pos = self.vertex_count;
        self.vertex_count += verts.len();
        let mut slice = &mut self.vertices[start_pos..self.vertex_count];
        for (i, vert) in slice.iter_mut().enumerate() {
            *vert = verts[i];
        }
    }
}

impl Default for VertexArray {
    fn default() -> Self {
        VertexArray {
            vertices: vec![Vertex::blank(); MAX_VERTICES].into_boxed_slice(),
            vertex_count: 0,
        }
    }
}
