use std::{
    cell::RefMut,
    sync::{Arc, Barrier, Mutex, RwLock, RwLockReadGuard},
};

use gamezap::{model::MeshManager, FrameDependancy};
use lazy_static::lazy_static;
use nalgebra as na;

use crate::{
    chunk::{BlockArray, Chunk},
    utils::{MeshTools, RingBuffer},
};

pub const RENDER_DISTANCE: usize = 3;

lazy_static! {
    pub static ref ALL_CHUNKS: Box<[BlockArray; RENDER_DISTANCE]> =
        Box::new([Chunk::default_blocks(); RENDER_DISTANCE]);
}

pub struct ChunkLoader {
    pub chunks: Arc<RwLock<RingBuffer<Chunk>>>,
    pub atlas_material_index: u32,
    pub center_chunk_position: na::Vector2<i32>,
    position_in_mesh_array: usize,
}

impl ChunkLoader {
    pub fn load_chunks(
        position: na::Vector2<f32>,
        atlas_material_index: u32,
        mesh_array_len: usize,
    ) -> Self {
        let chunked_position = position.map(|i| (i / 16.0) as i32);
        let origin_coords =
            na::Vector3::new(chunked_position.x as f32, 0.0, chunked_position.y as f32);
        ChunkLoader {
            chunks: Arc::new(RwLock::new(
                (0..RENDER_DISTANCE)
                    .map(|i| Chunk {
                        position: origin_coords.xz().map(|i| i as i32)
                            + na::Vector2::new(0, i as i32),
                        chunk_index: i,
                        atlas_material_index,
                    })
                    .collect::<RingBuffer<_>>(),
            )),
            atlas_material_index,
            center_chunk_position: chunked_position,
            position_in_mesh_array: mesh_array_len,
        }
    }
    pub fn reload_chunks(&mut self, offset: na::Vector2<i32>) {
        // if offset != na::Vector2::new(0, 0) {
        //     dbg!(offset);
        // }
        // let mut chunks_to_generate: Vec<na::Vector2<i32>> = vec![];
        if offset.y == -1 {
            let chunk_coords = self.chunks.read().unwrap()[-1].position + na::Vector2::new(0, 1);
            self.chunks.clone().write().unwrap().rotate_left(1);
            // chunks_to_generate.push(chunk_coords);
            self.chunks.clone().write().unwrap().replace_last(Chunk {
                position: chunk_coords,
                chunk_index: RENDER_DISTANCE - 1,
                atlas_material_index: self.atlas_material_index,
            })
        }
        // self.gen_chunk(&chunks_to_generate);
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
    pub fn render_chunks(
        chunks: &[Chunk],
        device: Arc<&wgpu::Device>,
        mesh_manager: &Arc<Mutex<MeshManager>>,
    ) {
        // let barrier = Arc::new(Barrier::new(chunks.len() + 1));
        // let chunks = chunks.clone();
        for chunk in chunks.iter() {
        // let barrier_clone = barrier.clone();
            chunk.create_mesh(device.clone(), mesh_manager.clone());
            // barrier_clone.wait();
        }
        // barrier.wait();
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
                let mesh_manager = renderer.module_manager.mesh_manager.as_ref().unwrap();
                self.reload_chunks(offset);

                let num_meshes = mesh_manager
                    .clone()
                    .lock()
                    .unwrap()
                    .diffuse_pipeline_models
                    .len();
                if num_meshes > self.position_in_mesh_array {
                    mesh_manager
                        .clone()
                        .lock()
                        .unwrap()
                        .diffuse_pipeline_models
                        .remove(self.position_in_mesh_array);
                    // mesh_manager.clone().lock().unwrap().diffuse_pipeline_models = vec![];
                    // .drain(num_meshes..num_meshes + RENDER_DISTANCE);
                    let device_arc = Arc::new(&renderer.device);
                    let chunks = [self.chunks.clone().read().unwrap()[-1]];
                    // std::thread::spawn(move || {
                    ChunkLoader::render_chunks(&chunks, device_arc.clone(), mesh_manager);
                    // });
                }
                self.center_chunk_position = chunked_position;
            }
        }
    }
}
