use std::{
    cell::{RefCell, RefMut},
    sync::{Arc, Mutex},
};

use gamezap::{model::MeshManager, FrameDependancy};
use lazy_static::lazy_static;
use nalgebra as na;
use threadpool::ThreadPool;

use crate::{
    chunk::{BlockArray, Chunk, X_SIZE, Y_SIZE, Z_SIZE},
    utils::MeshTools,
};

pub const RENDER_DISTANCE: usize = 3;

lazy_static! {
    static ref ALL_CHUNKS: Box<[BlockArray; RENDER_DISTANCE]> =
        Box::new([Chunk::default_blocks(); RENDER_DISTANCE]);
}

pub struct ChunkLoader {
    pub chunks: [Chunk; RENDER_DISTANCE],
    pub atlas_material_index: u32,
    pub center_chunk_position: na::Vector2<i32>,
}

impl ChunkLoader {
    pub fn load_chunks(position: na::Vector2<f32>, atlas_material_index: u32) -> Self {
        // let mut chunk_blocks = Box::new([[[1; Z_SIZE]; X_SIZE]; Y_SIZE]);
        // chunk_blocks[Y_SIZE - 1] = [[0; Z_SIZE]; X_SIZE];
        let chunked_position = position.map(|i| (i / 16.0) as i32);
        let origin_coords =
            na::Vector3::new(chunked_position.x as f32, 0.0, chunked_position.y as f32);
        ChunkLoader {
            chunks: (0..RENDER_DISTANCE)
                .map(|i| {
                    let chunk_ref: &'static BlockArray = &ALL_CHUNKS[i];
                    Chunk {
                        position: origin_coords.xz().map(|i| i as i32)
                            + na::Vector2::new(0, i as i32),
                        blocks: chunk_ref,
                        atlas_material_index,
                    }
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
        let mut chunks_to_generate: Vec<na::Vector2<i32>> = vec![];
        if offset.y == -1 {
            let chunk_coords = self.chunks[RENDER_DISTANCE - 1].position + na::Vector2::new(0, 1);
            self.chunks.rotate_left(1);
            chunks_to_generate.push(chunk_coords);
            let chunk_ref: &'static BlockArray = &ALL_CHUNKS[RENDER_DISTANCE - 1];
            self.chunks[RENDER_DISTANCE - 1] = Chunk {
                position: self.chunks[RENDER_DISTANCE - 2].position + na::Vector2::new(0, 1),
                blocks: chunk_ref,
                atlas_material_index: self.atlas_material_index,
            };
        }
        self.gen_chunk(&chunks_to_generate);
        // for chunk in generated_chunks {
        //     self.chunks[RENDER_DISTANCE - 1] = Chunk {
        //         position: *chunk.0,
        //         blocks: chunk.1,
        //         atlas_material_index: self.atlas_material_index,
        //     };
        // }
        self.center_chunk_position += offset;
    }

    fn gen_chunk<'a>(&mut self, chunks: &'a Vec<na::Vector2<i32>>) {
        // for chunk in chunks {
        //     let chunk_array_index = (chunk.y - self.chunks[0].position.y) as usize;
        //     let chunk_ref: &'static BlockArray = &ALL_CHUNKS[chunk_array_index];
        //     self.chunks[chunk_array_index] = Chunk {
        //         position: *chunk,
        //         blocks: chunk_ref,
        //         atlas_material_index: self.atlas_material_index,
        //     }
        // }
    }
    pub fn render_chunks(&self, device: &wgpu::Device, mesh_manager: &Arc<Mutex<MeshManager>>) {
        for chunk in &self.chunks {
            chunk.create_mesh(device, mesh_manager.clone());
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
                let renderer_borrow = &renderer.borrow();
                let mesh_manager = renderer_borrow
                    .module_manager
                    .mesh_manager
                    .as_ref()
                    .unwrap();
                self.reload_chunks(offset);

                let num_meshes = mesh_manager
                    .clone()
                    .lock()
                    .unwrap()
                    .diffuse_pipeline_models
                    .len();
                if num_meshes >= RENDER_DISTANCE {
                    mesh_manager.clone().lock().unwrap().diffuse_pipeline_models = vec![];
                    // .drain(num_meshes..num_meshes + RENDER_DISTANCE);
                    self.render_chunks(&renderer.borrow().device, mesh_manager);
                }
                self.center_chunk_position = chunked_position;
            }
        }
    }
}
