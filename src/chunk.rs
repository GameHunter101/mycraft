use std::{
    fmt::Debug,
    mem::transmute,
    sync::{
        atomic::{AtomicI64, AtomicUsize, Ordering::Relaxed},
        Arc, Barrier, Mutex,
    },
};

use gamezap::model::{Mesh, MeshTransform, Vertex};
use lazy_static::lazy_static;
use nalgebra as na;
use threadpool::ThreadPool;
use wgpu::util::DeviceExt;

use crate::{
    chunk_loader::{ALL_CHUNKS, RENDERED_CHUNKS_LENGTH},
    cube::{BlockWrapper, Blocks, Cube, FACE_INDICES},
};

pub const X_SIZE: usize = 16;
pub const Y_SIZE: usize = 256;
pub const Z_SIZE: usize = 16;

pub const HORIZONTAL_SLICE_SIZE: usize = X_SIZE * Z_SIZE;
pub const BLOCK_COUNT: usize = Z_SIZE * X_SIZE * Y_SIZE;

pub const MAX_VERTICES: usize = (24 * Z_SIZE * X_SIZE * Y_SIZE) / 2;
pub const MAX_INDICES: usize = (36 * Z_SIZE * X_SIZE * Y_SIZE) / 2;

lazy_static! {
    static ref ALL_INDICES: Box<[u32]> = (0..MAX_INDICES / 6)
        .flat_map(|face_index| FACE_INDICES.map(|i| i + (4 * face_index as u32)))
        .collect::<Vec<_>>()
        .into_boxed_slice();
}

pub type BlockArray = [u16; Z_SIZE * X_SIZE * Y_SIZE];

#[derive(Clone, Copy)]
pub struct Chunk {
    pub position: na::Vector2<i32>,
    pub chunk_index: (usize, usize),
    pub atlas_material_index: u32,
}

impl Chunk {
    fn query_block(chunk_index: usize, x: usize, y: usize, z: usize) -> Blocks {
        BlockWrapper[ALL_CHUNKS[chunk_index][y * HORIZONTAL_SLICE_SIZE + x * X_SIZE + z] as u32]
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
            if chunk_index > 0 && Self::query_block(chunk_index - 1, x, y, 0) == Blocks::Null {
                neighbors |= 0b0000_0001;
            }
        }
        if z < Z_SIZE - 1 {
            if Self::query_block(chunk_index, x, y, z + 1) == Blocks::Null {
                neighbors |= 0b0000_0010;
            }
        } else {
            if chunk_index < RENDERED_CHUNKS_LENGTH
                && Self::query_block(chunk_index + 1, x, y, Z_SIZE - 1) == Blocks::Null
            {
                neighbors |= 0b0000_0010;
            }
        }

        if x > 0 {
            if Self::query_block(chunk_index, x - 1, y, z) == Blocks::Null {
                neighbors |= 0b0000_0100;
            }
        } else {
            if chunk_index > RENDERED_CHUNKS_LENGTH
                && Self::query_block(chunk_index - RENDERED_CHUNKS_LENGTH, x, y, z) == Blocks::Null
            {
                neighbors |= 0b0000_0100;
            }
        }
        if x < X_SIZE - 1 {
            if Self::query_block(chunk_index, x + 1, y, z) == Blocks::Null {
                neighbors |= 0b0000_1000;
            }
        } else {
            if chunk_index < RENDERED_CHUNKS_LENGTH * (RENDERED_CHUNKS_LENGTH - 1)
                && Self::query_block(chunk_index + RENDERED_CHUNKS_LENGTH, x, y, z) == Blocks::Null
            {
                neighbors |= 0b0000_1000;
            }
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
            neighbors |= 0b0010_0000;
        }
        neighbors
    }

    pub const fn default_blocks() -> BlockArray {
        let mut blocks = [[[1_u16; Z_SIZE]; X_SIZE]; Y_SIZE];
        blocks[Y_SIZE - 1] = [[0; Z_SIZE]; X_SIZE];
        unsafe { std::mem::transmute(blocks) }
    }

    pub fn gen_block_vertices<'a>(
        chunk_index: (usize, usize),
        x: usize,
        y: usize,
        z: usize,
    ) -> Vec<Vertex> {
        let chunk_index = chunk_index.0 * RENDERED_CHUNKS_LENGTH + chunk_index.1;
        let block_type = Self::query_block(chunk_index, x, y, z);

        if block_type != Blocks::Null {
            let face_mask = Self::query_neighbors(chunk_index, x, y, z);
            if face_mask != 0 {
                let block = Cube::new(
                    na::Vector3::new(x as f32, y as f32, z as f32),
                    0,
                    block_type,
                    face_mask,
                    true,
                );

                return (&block.mesh_info.vertices[..block.mesh_info.vertex_count]).to_vec();
                // vertices_clone.lock().unwrap().push(
                //     &block.mesh_info.vertices
                //         [..block.mesh_info.vertex_count],
                // );
            } else {
                return vec![];
            }
        } else {
            return vec![];
        }
    }

    pub fn create_mesh(&self, device: Arc<wgpu::Device>) -> Arc<Mesh> {
        // let pool = rayon::ThreadPoolBuilder::new()
        //     .num_threads(8)
        //     .build()
        //     .unwrap();
        let start_time = time::Instant::now();
        let pool = ThreadPool::new(30);

        let vertices_2d: Arc<Mutex<Vec<Vec<Vertex>>>> =
            Arc::new(Mutex::new(Vec::with_capacity(MAX_VERTICES)));
        let vertices_count = Arc::new(AtomicUsize::new(0));

        let vertices_count_clone = vertices_count.clone();
        let total_time = Arc::new(AtomicI64::new(0));
        (0..BLOCK_COUNT).into_iter().for_each(|block_index| {
            let total_time = total_time.clone();
            let vertices_2d = vertices_2d.clone();
            let chunk_index = self.chunk_index;
            let vertices_count = vertices_count.clone();
            pool.execute(move || {
                let start_time = time::Instant::now();
                let y_pos = block_index / HORIZONTAL_SLICE_SIZE;
                let x_pos = block_index % HORIZONTAL_SLICE_SIZE / X_SIZE;
                let z_pos = block_index % HORIZONTAL_SLICE_SIZE % X_SIZE;

                let verts = Self::gen_block_vertices(chunk_index, x_pos, y_pos, z_pos);
                vertices_2d.lock().unwrap().push(verts);
                vertices_count.fetch_add(1, Relaxed);
                let current_elapsed_time =
                    (time::Instant::now() - start_time).whole_nanoseconds() as i64;
                // println!("current elapsed: {}", current_elapsed_time);
                total_time.fetch_add(current_elapsed_time, Relaxed);
            });
        });

        let position_x = self.position.x as f32;
        let position_y = self.position.y as f32;
        std::thread::spawn(move || loop {
            let vertices_count = vertices_count_clone.clone().load(Relaxed);
            if vertices_count == BLOCK_COUNT {
                let chunk_generation_time = time::Instant::now();
                let average_time = (total_time.load(Relaxed) / BLOCK_COUNT as i64) as f32 / 1000000.0;
                println!(
                    "Chunk generation time: {}, average chunk time: {}, total chunk time: {}",
                    (chunk_generation_time - start_time).whole_milliseconds(),
                    average_time,
                    total_time.load(Relaxed) as f32 / 1000000.0
                );
                let vertices = vertices_2d.lock().unwrap()[..vertices_count].concat();
                let vertices_len = vertices_count;

                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Chunk 1 vert buff"),
                    usage: wgpu::BufferUsages::VERTEX,
                    contents: bytemuck::cast_slice(&vertices),
                });

                let total_index_count = (vertices_len as f32 * 1.5) as usize;
                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Chunk 1 index buff"),
                    usage: wgpu::BufferUsages::INDEX,
                    contents: bytemuck::cast_slice(&ALL_INDICES[..total_index_count]),
                });

                let mesh = Arc::new(Mesh::new(
                    &device,
                    "Chunk 1".to_string(),
                    vertex_buffer,
                    index_buffer,
                    total_index_count as u32,
                    MeshTransform::new(
                        na::Vector3::new(position_x * 16.0, 0.0, position_y * 16.0),
                        na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), 0.0),
                    ),
                    0,
                ));
                return mesh;
            }
        })
        .join()
        .unwrap()
    }
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x: {}, y: {}", self.position.x, self.position.y)
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
        let slice = &mut self.vertices[start_pos..self.vertex_count];
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
