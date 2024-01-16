use std::{
    fmt::Debug,
    sync::{
        atomic::{AtomicUsize, Ordering::Relaxed},
        Arc, Mutex,
    },
};

use gamezap::model::{Mesh, MeshTransform, Vertex};
use lazy_static::lazy_static;
use nalgebra as na;
use threadpool::ThreadPool;
use wgpu::util::DeviceExt;

use crate::{
    chunk_loader::{ALL_BLOCK_STATES, RENDERED_CHUNKS_LENGTH},
    cube::{BlockWrapper, Blocks, Cube, MeshInfo, FACE_INDICES},
};

pub const X_SIZE: usize = 16;
pub const Y_SIZE: usize = 256;
pub const Z_SIZE: usize = 16;

pub const HORIZONTAL_SLICE_SIZE: usize = X_SIZE * Z_SIZE;
pub const BLOCK_COUNT: usize = Z_SIZE * X_SIZE * Y_SIZE;

pub const MAX_VERTICES: usize = (24 * Z_SIZE * X_SIZE * Y_SIZE) / 2;
pub const MAX_INDICES: usize = (36 * Z_SIZE * X_SIZE * Y_SIZE) / 2;

pub const BLOCK_GROUP_SIZE: usize = 1;

lazy_static! {
    static ref ALL_INDICES: Box<[u32]> = (0..MAX_INDICES / 6)
        .flat_map(|face_index| FACE_INDICES.map(|i| i + (4 * face_index as u32)))
        .collect::<Vec<_>>()
        .into_boxed_slice();
}

pub type BlockArray = [u16; Z_SIZE * X_SIZE * Y_SIZE];

pub struct Chunk {
    pub position: na::Vector2<i32>,
    pub chunk_index: (u32, u32),
    pub atlas_material_index: u32,

}

impl Chunk {
    pub fn new(
        position: na::Vector2<i32>,
        chunk_index: (u32, u32),
        atlas_material_index: u32,
    ) -> Self {
        Chunk {
            position,
            chunk_index,
            atlas_material_index,
        }
    }

    /* pub fn gen_verts(&self) -> Vec<Face> {
        let chunk_block_states = ALL_BLOCK_STATES[self.chunk_index.0][self.chunk_index.1];
        let material_index = self.atlas_material_index;
        let chunk_index = self.chunk_index;
        let mut chunk_face_data: Vec<Face> = Vec::with_capacity(MAX_VERTICES);

        for (block_index, block) in chunk_block_states.iter().enumerate() {
            let y = block_index / HORIZONTAL_SLICE_SIZE;
            let x = (block_index - y * HORIZONTAL_SLICE_SIZE) / X_SIZE;
            let z = block_index - x * X_SIZE - y * Y_SIZE;

            let block_position = na::Vector3::new(x as f32, y as f32, z as f32);
            let face_mask = Chunk::query_neighbors(chunk_index, x, y, z);

            let block = Cube::new(
                block_position,
                material_index,
                BlockWrapper[*block],
                face_mask,
                true,
            );
            for face in block.mesh_info.faces {
                chunk_face_data.push(face.into());
            }
        }
        chunk_face_data
    }

    pub fn gen_mesh(&self, device: Arc<wgpu::Device>) -> Mesh {
        let vertices = self.gen_verts();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk 1 vert buff"),
            usage: wgpu::BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(&vertices),
        });

        let total_index_count = (vertices.len() as f32 * 1.5) as usize;
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk 1 index buff"),
            usage: wgpu::BufferUsages::INDEX,
            contents: bytemuck::cast_slice(&ALL_INDICES[..total_index_count]),
        });

        let mesh = Mesh::new(
            &device,
            format!("Chunk {}, {}", self.chunk_index.0, self.chunk_index.1),
            vertex_buffer,
            index_buffer,
            total_index_count as u32,
            MeshTransform::new(
                na::Vector3::new(0.0, 0.0, 0.0),
                na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), 0.0),
            ),
            self.atlas_material_index,
        );

        mesh
    }*/

    fn query_block(chunk_index: (u32, u32), x: usize, y: usize, z: usize) -> Blocks {
        let chunk_block_states = &ALL_BLOCK_STATES[chunk_index.0][chunk_index.1];
        BlockWrapper[chunk_block_states[y * HORIZONTAL_SLICE_SIZE + x * X_SIZE + z]]
    }

    /// Returns a bitmask of which faces have neighboring blocks
    ///
    /// From right to left: `1: -z, 2: +z, 4: -x, 8: +x, 16: +y, 32: -y`
    fn query_neighbors(chunk_index: (u32, u32), x: usize, y: usize, z: usize) -> u8 {
        let mut neighbors = 0b0000_0000;

        if z > 0 {
            if Self::query_block(chunk_index, x, y, z - 1) == Blocks::Null {
                neighbors |= 0b0000_0001;
            }
        } else {
            if chunk_index.1 > 0
                && Self::query_block((chunk_index.0, chunk_index.1 - 1), x, y, Z_SIZE - 1)
                    == Blocks::Null
            {
                neighbors |= 0b0000_0001;
            }
        }
        if z < Z_SIZE - 1 {
            if Self::query_block(chunk_index, x, y, z + 1) == Blocks::Null {
                neighbors |= 0b0000_0010;
            }
        } else {
            if chunk_index.1 < RENDERED_CHUNKS_LENGTH as u32 - 1
                && Self::query_block((chunk_index.0, chunk_index.1 + 1), x, y, 0) == Blocks::Null
            {
                neighbors |= 0b0000_0010;
            }
        }

        if x > 0 {
            if Self::query_block(chunk_index, x - 1, y, z) == Blocks::Null {
                neighbors |= 0b0000_0100;
            }
        } else {
            if chunk_index.0 > 0
                && Self::query_block((chunk_index.0 - 1, chunk_index.1), X_SIZE - 1, y, z)
                    == Blocks::Null
            {
                neighbors |= 0b0000_0100;
            }
        }
        if x < X_SIZE - 1 {
            if Self::query_block(chunk_index, x + 1, y, z) == Blocks::Null {
                neighbors |= 0b0000_1000;
            }
        } else {
            if chunk_index.0 < RENDERED_CHUNKS_LENGTH as u32 - 1
                && Self::query_block((chunk_index.0 + 1, chunk_index.1), 0, y, z) == Blocks::Null
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
        chunk_index: (u32, u32),
        x: usize,
        y: usize,
        z: usize,
    ) -> MeshInfo {
        // let chunk_index = chunk_index.0 * RENDERED_CHUNKS_LENGTH + chunk_index.1;
        let block_type = Self::query_block(chunk_index, x, y, z);

        if block_type != Blocks::Null {
            let face_mask = Self::query_neighbors(chunk_index, x, y, z);
            if face_mask != 0 {
                // return MeshInfo::full(na::Vector3::new(x as f32, y as f32, z as f32));
                let block = Cube::new(
                    na::Vector3::new(x as f32, y as f32, z as f32),
                    0,
                    block_type,
                    face_mask,
                    true,
                );

                return block.mesh_info;
                // vertices_clone.lock().unwrap().push(
                //     &block.mesh_info.vertices
                //         [..block.mesh_info.vertex_count],
                // );
            } else {
                return MeshInfo::init();
            }
        } else {
            return MeshInfo::init();
        }
    }

    pub fn create_mesh(&self, device: Arc<wgpu::Device>) -> Arc<Mesh> {
        // let pool = rayon::ThreadPoolBuilder::new()
        //     .num_threads(8)
        //     .build()
        //     .unwrap();
        let pool = ThreadPool::new(6);

        let vertices_2d: Arc<Mutex<Vec<[Vertex; 4]>>> =
            Arc::new(Mutex::new(Vec::with_capacity(MAX_VERTICES)));
        let faces_count = Arc::new(AtomicUsize::new(0));

        let faces_count_clone = faces_count.clone();
        (0..BLOCK_COUNT / BLOCK_GROUP_SIZE)
            .into_iter()
            .for_each(|blocks_group_start| {
                let vertices_2d = vertices_2d.clone();
                let chunk_index = self.chunk_index;
                let faces_count = faces_count.clone();
                // pool.execute(move || {
                for block_index in blocks_group_start * BLOCK_GROUP_SIZE
                    ..(blocks_group_start + 1) * BLOCK_GROUP_SIZE
                {
                    let y_pos = block_index / HORIZONTAL_SLICE_SIZE;
                    let x_pos = block_index % HORIZONTAL_SLICE_SIZE / X_SIZE;
                    let z_pos = block_index % HORIZONTAL_SLICE_SIZE % X_SIZE;

                    let faces_info = Self::gen_block_vertices(chunk_index, x_pos, y_pos, z_pos);
                    for face in &faces_info.faces[..faces_info.face_count] {
                        vertices_2d.lock().unwrap().push(*face);
                        faces_count.fetch_add(1, Relaxed);
                    }
                }
                // })
            });
        pool.join();

        let faces_count = faces_count_clone.clone().load(Relaxed);
        let vertices_count = faces_count * 4;

        let position_x = self.position.x as f32;
        let position_y = self.position.y as f32;
        let vertices = vertices_2d.lock().unwrap()[..faces_count].concat();
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

        mesh
    }
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x: {}, y: {}", self.position.x, self.position.y)
    }
}
