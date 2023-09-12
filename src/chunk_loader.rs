use std::cell::{RefCell, RefMut};

use gamezap::{model::MeshManager, FrameDependancy};
use nalgebra as na;

use crate::{
    chunk::{Chunk, X_SIZE, Y_SIZE, Z_SIZE},
    utils::MeshTools,
};

pub const RENDER_DISTANCE: usize = 10;
pub struct ChunkLoader {
    pub chunks: [Chunk; RENDER_DISTANCE],
    pub atlas_material_index: u32,
    pub center_chunk_position: na::Vector2<i32>,
}

impl ChunkLoader {
    pub fn load_chunks(position: na::Vector2<f32>, atlas_material_index: u32) -> Self {
        let mut chunk_blocks = Box::new([[[1; Z_SIZE]; X_SIZE]; Y_SIZE]);
        chunk_blocks[Y_SIZE - 1] = [[0; Z_SIZE]; X_SIZE];
        let chunked_position = position.map(|i| (i / 16.0) as i32);
        let origin_coords =
            na::Vector3::new(chunked_position.x as f32, 0.0, chunked_position.y as f32);
        ChunkLoader {
            chunks: (0..RENDER_DISTANCE)
                .map(|i| Chunk {
                    position: origin_coords + na::Vector3::new(0.0, 0.0, 16.0 * i as f32),
                    blocks: chunk_blocks.clone(),
                    atlas_material_index,
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            atlas_material_index,
            center_chunk_position: chunked_position,
        }
    }
    pub fn reload_chunks(&mut self, offset: na::Vector2<i32>) {
        // if offset != na::Vector2::new(0, 0) {
        //     dbg!(offset);
        // }
        if offset.y == -1 {
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

impl FrameDependancy for ChunkLoader {
    fn frame_update(
        &mut self,
        _engine_details: RefMut<gamezap::EngineDetails>,
        renderer: &RefCell<gamezap::renderer::Renderer>,
        _engine_systems: std::cell::Ref<gamezap::EngineSystems>,
    ) {
        let camera_manager = &renderer.borrow().module_manager.camera_manager;
        if let Some(camera_manager) = camera_manager {
            let position = camera_manager.borrow().camera.borrow().position.xz();
            let chunked_position = position.map(|i| i as i32 / 16);
            let offset = chunked_position - self.center_chunk_position;
            if offset != na::Vector2::new(0, 0) {
                self.reload_chunks(offset);
                self.render_chunks(
                    &renderer.borrow().device,
                    &renderer
                        .borrow()
                        .module_manager
                        .mesh_manager
                        .as_ref()
                        .unwrap(),
                );
                self.center_chunk_position = chunked_position;
            }
        }
    }
}
