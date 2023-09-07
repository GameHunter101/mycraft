use std::{cell::{RefMut, RefCell}, borrow::BorrowMut};

use gamezap::model::MeshManager;
use nalgebra as na;

use crate::{
    chunk::{Chunk, X_SIZE, Y_SIZE, Z_SIZE},
    utils::MeshTools,
};

pub const RENDER_DISTANCE: usize = 10;
pub struct ChunkLoader {
    pub chunks: [Chunk; RENDER_DISTANCE],
    pub atlas_material_index: u32,
}

impl ChunkLoader {
    pub fn load_chunks(position: na::Vector3<f32>, atlas_material_index: u32) -> Self {
        let mut chunk_blocks = Box::new([[[1; Z_SIZE]; X_SIZE]; Y_SIZE]);
        chunk_blocks[Y_SIZE - 1] = [[0; Z_SIZE]; X_SIZE];
        ChunkLoader {
            chunks: (0..RENDER_DISTANCE)
                .map(|i| Chunk {
                    position: position + na::Vector3::new(0.0, 0.0, 16.0 * i as f32),
                    blocks: chunk_blocks.clone(),
                    atlas_material_index,
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            atlas_material_index,
        }
    }
    pub fn reload_chunks(&mut self, offset: na::Vector3<f32>) {
        if offset.z == 1.0 {
            self.chunks.rotate_left(1);
            self.chunks[RENDER_DISTANCE - 1] = Chunk {
                position: self.chunks[RENDER_DISTANCE - 2].position
                    + na::Vector3::new(0.0, 0.0, 16.0),
                blocks: self.chunks[RENDER_DISTANCE - 2].blocks.clone(),
                atlas_material_index: self.atlas_material_index,
            };
        }
    }
    pub fn render_chunks(&self, device: &wgpu::Device, mesh_manager: &RefCell<MeshManager>) {
        for chunk in &self.chunks {
            chunk.create_mesh(device, mesh_manager.borrow_mut());
        }
    }
}
