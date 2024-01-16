use std::{
    borrow::Borrow,
    cell::RefMut,
    sync::{Arc, Mutex},
};

use gamezap::{model::MeshManager, FrameDependancy};
use lazy_static::lazy_static;
use nalgebra as na;

use crate::{
    chunk::{BlockArray, Chunk},
    ring_buffer::RingBuffer2D,
};

pub const RENDER_DISTANCE: usize = 1;
pub const RENDERED_CHUNKS_LENGTH: usize = 2 * RENDER_DISTANCE + 1;

lazy_static! {
    pub static ref ALL_BLOCK_STATES: RingBuffer2D<BlockArray> = RingBuffer2D::new(
        (0..RENDERED_CHUNKS_LENGTH)
            .map(|_| (0..RENDERED_CHUNKS_LENGTH)
                .map(|_| Chunk::default_blocks())
                .collect::<Vec<_>>())
            .collect::<Vec<_>>()
    );
}

/// Chunks [RingBuffer2D] is shaped like this:
/// .| | | | |
/// .| | | | |
/// .| | | | |
/// .| | | | |
/// .| | | | |
/// (0,0) of the chunks [RingBuffer2D] corresponds to (-[RENDER_DISTANCE], -[RENDER_DISTANCE])
pub struct ChunkLoader {
    pub chunks: RingBuffer2D<Arc<Mutex<Chunk>>>,
    pub atlas_material_index: u32,
    pub center_chunk_position: na::Vector2<i32>,
    position_in_mesh_array: usize,
}

impl ChunkLoader {
    pub fn new(atlas_material_index: u32, current_mesh_count: usize) -> Self {
        let chunks = (-(RENDER_DISTANCE as i32)..=RENDER_DISTANCE as i32)
            .map(|x| {
                (-(RENDER_DISTANCE as i32)..=RENDER_DISTANCE as i32)
                    .map(|y| {
                        Arc::new(Mutex::new(Chunk::new(
                            na::Vector2::new(x, y),
                            (
                                (x + RENDER_DISTANCE as i32) as u32,
                                (y + RENDER_DISTANCE as i32) as u32,
                            ),
                            atlas_material_index,
                        )))
                    })
                    .collect::<Vec<Arc<Mutex<Chunk>>>>()
            })
            .collect::<Vec<Vec<Arc<Mutex<Chunk>>>>>();
        let chunks = RingBuffer2D::new(chunks);

        Self {
            chunks,
            atlas_material_index,
            center_chunk_position: na::Vector2::new(0, 0),
            position_in_mesh_array: current_mesh_count,
        }
    }

    pub fn initialize_chunks(
        &self,
        mesh_manager: Arc<Mutex<MeshManager>>,
        device: Arc<wgpu::Device>,
    ) {
        for chunk_column in &self.chunks {
            for chunk in chunk_column.into_iter() {
                let chunk_mesh = chunk.clone().lock().unwrap().create_mesh(device.clone());
                mesh_manager
                    .lock()
                    .unwrap()
                    .diffuse_pipeline_models
                    .push(chunk_mesh);
            }
        }
    }

    fn mark_chunks_to_reload(&self, offset: (i32, i32)) -> Vec<(i32, i32)> {
        match offset {
            (-1, 0) => {
                return (0..RENDERED_CHUNKS_LENGTH as i32)
                    .map(|y| (0, y))
                    .collect::<Vec<(i32, i32)>>()
            }
            (1, 0) => {
                return (0..RENDERED_CHUNKS_LENGTH as i32)
                    .map(|y| (-1, y))
                    .collect::<Vec<(i32, i32)>>()
            }
            (0, 1) => {
                return (0..RENDERED_CHUNKS_LENGTH as i32)
                    .map(|x| (x, -1))
                    .collect::<Vec<(i32, i32)>>()
            }
            (0, -1) => {
                return (0..RENDERED_CHUNKS_LENGTH as i32)
                    .map(|x| (x, 0))
                    .collect::<Vec<(i32, i32)>>()
            }
            _ => vec![],
        }
    }

    fn create_new_chunk_meshes(
        &mut self,
        offset: (i32, i32),
        chunk_indices: Vec<(i32, i32)>,
        device: Arc<wgpu::Device>,
        mesh_manager: Arc<Mutex<MeshManager>>,
    ) {
        for chunk_index in chunk_indices {
            let mut current_chunk = self.chunks[chunk_index].lock().unwrap();
            if offset.0 != 0 {
                current_chunk.position.x =
                    -1 * self.center_chunk_position.x - offset.0 * (RENDER_DISTANCE as i32 + 1);
            }

            if offset.1 != 0 {
                current_chunk.position.y =
                    -1 * self.center_chunk_position.y - offset.1 * (RENDER_DISTANCE as i32 + 1);
            }

            let new_mesh = current_chunk.create_mesh(device.clone());
            let linearized_index = self.position_in_mesh_array
                + match offset {
                    (-1, 0) => self.chunks.linearize_index((0, chunk_index.1)),
                    (1, 0) => self.chunks.linearize_index((-1, chunk_index.1)),
                    (0, 1) => self.chunks.linearize_index((chunk_index.0, -1)),
                    (0, -1) => self.chunks.linearize_index((chunk_index.0, 0)),
                    _ => 0,
                };
            mesh_manager.lock().unwrap().diffuse_pipeline_models[linearized_index] = new_mesh;
        }
        match offset {
            (-1, 0) => self.chunks.rotate_left(1),
            (1, 0) => self.chunks.rotate_right(1),
            (0, 1) => self.chunks.rotate_up(1),
            (0, -1) => self.chunks.rotate_down(1),
            _ => {}
        };
    }
}

impl FrameDependancy for ChunkLoader {
    fn frame_update(
        &mut self,
        _engine_details: RefMut<gamezap::EngineDetails>,
        renderer: &gamezap::renderer::Renderer,
        _engine_systems: std::cell::Ref<gamezap::EngineSystems>,
    ) {
        let camera_manager = renderer
            .module_manager
            .camera_manager
            .as_ref()
            .unwrap()
            .borrow();
        let position = camera_manager.camera.borrow().position;
        let chunked_position = na::Vector2::new(position.x as i32 / 16, position.z as i32 / 16);

        let offset_vec = chunked_position - self.center_chunk_position;
        let offset = (offset_vec.x, offset_vec.y);

        if offset != (0, 0) {
            let chunks_to_load = self.mark_chunks_to_reload(offset);
            let mesh_manager = renderer
                .module_manager
                .borrow()
                .mesh_manager
                .borrow()
                .as_ref()
                .unwrap();
            self.create_new_chunk_meshes(
                offset,
                chunks_to_load,
                renderer.device.clone(),
                mesh_manager.clone(),
            );
        }
        self.center_chunk_position = chunked_position;
    }
}
