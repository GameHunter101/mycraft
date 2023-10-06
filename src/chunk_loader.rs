use std::{
    cell::RefMut,
    sync::{Arc, Barrier, Mutex},
};

use gamezap::{model::Mesh, FrameDependancy};
use lazy_static::lazy_static;
use nalgebra as na;

use crate::{
    chunk::{BlockArray, Chunk},
    ring_buffer::{RingBuffer, RingBuffer2D},
};

pub const RENDER_DISTANCE: usize = 1;
pub const RENDERED_CHUNKS_LENGTH: usize = 2 * RENDER_DISTANCE + 1;

lazy_static! {
    pub static ref ALL_CHUNKS: Box<[BlockArray]> =
        vec![Chunk::default_blocks(); RENDERED_CHUNKS_LENGTH * RENDERED_CHUNKS_LENGTH]
            .into_boxed_slice();
}

pub struct ChunkLoader {
    pub chunks: RingBuffer2D<Arc<Chunk>>,
    pub atlas_material_index: u32,
    pub center_chunk_position: na::Vector2<i32>,
    position_in_mesh_array: usize,
    chunk_meshes: RingBuffer2D<Option<Arc<Mesh>>>,
    is_pipeline_updated: bool,
}

impl ChunkLoader {
    pub fn load_chunks(
        position: na::Vector2<f32>,
        atlas_material_index: u32,
        mesh_array_len: usize,
        device: Arc<wgpu::Device>,
    ) -> Self {
        let chunked_position = position.map(|i| (i / 16.0) as i32);
        let origin_coords =
            na::Vector3::new(chunked_position.x as f32, 0.0, chunked_position.y as f32);
        let chunks = (0..RENDERED_CHUNKS_LENGTH)
            .map(|x| {
                (0..RENDERED_CHUNKS_LENGTH)
                    .map(|z| {
                        Arc::new(Chunk {
                            position: origin_coords.xz().map(|i| i as i32)
                                + na::Vector2::new(
                                    x as i32 - RENDER_DISTANCE as i32,
                                    z as i32 - RENDER_DISTANCE as i32,
                                ),
                            chunk_index: (x, z),
                            atlas_material_index,
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let chunk_meshes = (&chunks)
            .into_iter()
            .map(|z_slice| {
                ChunkLoader::render_chunks(z_slice.to_vec(), device.clone())
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|c| c.clone())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        ChunkLoader {
            chunks: RingBuffer2D::new(chunks),
            atlas_material_index,
            center_chunk_position: chunked_position,
            position_in_mesh_array: mesh_array_len,
            chunk_meshes: RingBuffer2D::new(chunk_meshes),
            is_pipeline_updated: false,
        }
    }

    pub fn reload_chunks(&mut self, offset: na::Vector2<i32>, device: Arc<wgpu::Device>) {
        // Moving positive z
        if offset.y == -1 {
            let mut original_chunks = Vec::with_capacity(RENDERED_CHUNKS_LENGTH);
            let chunks_to_render: Vec<Arc<Chunk>> = self
                .chunks
                .index_horizontal(-1)
                .into_iter()
                .map(|c| {
                    original_chunks.push(c.clone());
                    let chunk_coords = c.position + na::Vector2::new(0, 1);
                    Arc::new(Chunk {
                        position: chunk_coords,
                        chunk_index: (0, RENDER_DISTANCE - 1),
                        atlas_material_index: self.atlas_material_index,
                    })
                })
                .collect();
            self.chunks.rotate_down(1);
            self.chunks.mut_index_horizontal(-1, &chunks_to_render);

            let new_chunks = ChunkLoader::render_chunks(chunks_to_render, device.clone());
            self.chunk_meshes.rotate_down(1);
            self.chunk_meshes
                .mut_index_horizontal(-1, &new_chunks.lock().unwrap());
        }
        // Moving negatve z
        if offset.y == 1 {
            let chunks_to_render: Vec<Arc<Chunk>> = self
                .chunks
                .index_horizontal(0)
                .iter()
                .map(|c| {
                    let chunk_coords = c.position + na::Vector2::new(0, -1);
                    Arc::new(Chunk {
                        position: chunk_coords,
                        chunk_index: (0, RENDER_DISTANCE - 1),
                        atlas_material_index: self.atlas_material_index,
                    })
                })
                .collect();
            self.chunks.rotate_up(1);
            self.chunks.mut_index_horizontal(0, &chunks_to_render);

            let new_chunks = ChunkLoader::render_chunks(chunks_to_render, device.clone());
            self.chunk_meshes.rotate_up(1);
            self.chunk_meshes
                .mut_index_horizontal(0, &new_chunks.lock().unwrap());
        }

        // Moving positive x
        if offset.x == -1 {
            let chunks_to_render: Vec<Arc<Chunk>> = self.chunks[-1]
                .into_iter()
                .map(|c| {
                    let chunk_coords = c.position + na::Vector2::new(1, 0);
                    Arc::new(Chunk {
                        position: chunk_coords,
                        chunk_index: (RENDER_DISTANCE - 1, 0),
                        atlas_material_index: self.atlas_material_index,
                    })
                })
                .collect();
            self.chunks.rotate_left(1);
            self.chunks.replace_last(RingBuffer::new(
                (&chunks_to_render).into_iter().map(|c| c.clone()).collect(),
            ));

            let new_chunks = ChunkLoader::render_chunks(chunks_to_render, device.clone());
            self.chunk_meshes.rotate_left(1);

            for (i, chunk) in (&mut *new_chunks.clone().lock().unwrap())
                .into_iter()
                .enumerate()
            {
                self.chunk_meshes[-1][i as i32] = chunk.clone();
            }
        }

        // Moving negative x
        if offset.x == 1 {
            let chunks_to_render: Vec<Arc<Chunk>> = self.chunks[0]
                .into_iter()
                .map(|c| {
                    let chunk_coords = c.position + na::Vector2::new(-1, 0);
                    Arc::new(Chunk {
                        position: chunk_coords,
                        chunk_index: (RENDER_DISTANCE - 1, 0),
                        atlas_material_index: self.atlas_material_index,
                    })
                })
                .collect();
            self.chunks.rotate_right(1);
            self.chunks.replace_first(RingBuffer::new(
                (&chunks_to_render).into_iter().map(|c| c.clone()).collect(),
            ));

            let new_chunks = ChunkLoader::render_chunks(chunks_to_render, device.clone());
            self.chunk_meshes.rotate_right(1);

            for (i, chunk) in (&mut *new_chunks.clone().lock().unwrap())
                .into_iter()
                .enumerate()
            {
                self.chunk_meshes[0][i as i32] = chunk.clone();
            }
        }
    }

    pub fn render_chunks(
        chunks: Vec<Arc<Chunk>>,
        device: Arc<wgpu::Device>,
    ) -> Arc<Mutex<Vec<Option<Arc<Mesh>>>>> {
        let barrier = Arc::new(Barrier::new(chunks.len() + 1));
        let meshes: Arc<Mutex<Vec<Option<Arc<Mesh>>>>> =
            Arc::new(Mutex::new(vec![None; chunks.len()]));
        for (i, chunk_data) in chunks.iter().enumerate() {
            let chunk = chunk_data.clone();
            let device = device.clone();
            let barrier = barrier.clone();
            let meshes_clone = meshes.clone();
            std::thread::spawn(move || {
                let mesh = chunk.create_mesh(device);
                meshes_clone.clone().lock().unwrap()[i] = Some(mesh.clone());
                barrier.wait();
            });
        }
        barrier.clone().wait();
        meshes
    }
}

impl FrameDependancy for ChunkLoader {
    fn frame_update(
        &mut self,
        _engine_details: RefMut<gamezap::EngineDetails>,
        renderer: &gamezap::renderer::Renderer,
        _engine_systems: std::cell::Ref<gamezap::EngineSystems>,
    ) {
        let camera_manager = &renderer.module_manager.camera_manager;
        if let Some(camera_manager) = camera_manager {
            let position = camera_manager.borrow().camera.borrow().position.xz();
            let chunked_position = position.map(|i| i as i32 / 16);
            let offset = chunked_position - self.center_chunk_position;
            if offset != na::Vector2::new(0, 0) {
                let device_arc = renderer.device.clone();
                self.reload_chunks(offset, device_arc);
                self.is_pipeline_updated = false;
            }
            if !self.is_pipeline_updated {
                let mesh_manager = renderer
                    .module_manager
                    .mesh_manager
                    .as_ref()
                    .unwrap()
                    .clone();

                let chunk_meshes = self.chunk_meshes.flatten();

                let num_meshes = mesh_manager
                    .clone()
                    .lock()
                    .unwrap()
                    .diffuse_pipeline_models
                    .len() as i32;
                for (i, mesh) in chunk_meshes.iter().enumerate() {
                    if num_meshes - RENDERED_CHUNKS_LENGTH as i32 > 0 {
                        mesh_manager.clone().lock().unwrap().diffuse_pipeline_models
                            [i + self.position_in_mesh_array] = mesh.as_ref().unwrap().clone();
                    } else {
                        mesh_manager
                            .clone()
                            .lock()
                            .unwrap()
                            .diffuse_pipeline_models
                            .push(mesh.as_ref().unwrap().clone());
                    }
                }

                self.is_pipeline_updated = true;
                self.center_chunk_position = chunked_position;
            }
        }
    }
}
