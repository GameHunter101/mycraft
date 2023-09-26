use std::{
    cell::RefMut,
    sync::{Arc, Barrier, Mutex, RwLock},
};

use gamezap::{
    model::{Mesh, MeshManager},
    FrameDependancy,
};
use lazy_static::lazy_static;
use nalgebra as na;
use threadpool::ThreadPool;

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
    pub chunks: RingBuffer<Arc<Chunk>>,
    pub atlas_material_index: u32,
    pub center_chunk_position: na::Vector2<i32>,
    position_in_mesh_array: usize,
    chunk_meshes: RingBuffer<Option<Arc<Mesh>>>,
    is_pipeline_updated: bool,
}

impl ChunkLoader {
    pub fn load_chunks(
        position: na::Vector2<f32>,
        atlas_material_index: u32,
        mesh_array_len: usize,
        device: Arc<wgpu::Device>,
        mesh_manager: Arc<Mutex<MeshManager>>,
    ) -> Self {
        let chunked_position = position.map(|i| (i / 16.0) as i32);
        let origin_coords =
            na::Vector3::new(chunked_position.x as f32, 0.0, chunked_position.y as f32);
        let chunk_start = 0; //-(RENDER_DISTANCE as i32 / 2);
        let chunks = (0..RENDER_DISTANCE)
            .map(|i| {
                Arc::new(Chunk {
                    position: origin_coords.xz().map(|i| i as i32)
                        + na::Vector2::new(0, i as i32 + chunk_start),
                    chunk_index: i,
                    atlas_material_index,
                })
            })
            .collect::<RingBuffer<_>>();
        // for chunk in &chunks.buffer[..] {
        //     println!("chunk pos: {}", chunk.clone().position);
        // }
        let chunk_meshes = ChunkLoader::render_chunks(chunks.buffer.to_vec(), device, mesh_manager)
            .lock()
            .unwrap()
            .iter()
            .map(|c| c.clone())
            .collect::<RingBuffer<_>>();
        ChunkLoader {
            chunks,
            atlas_material_index,
            center_chunk_position: chunked_position,
            position_in_mesh_array: mesh_array_len,
            chunk_meshes,
            is_pipeline_updated: false,
        }
    }

    fn debug_chunk_positions(&self) -> String {
        let chunks = &self.chunks.buffer;
        format!(
            "{:?}, {:?}, {:?}",
            chunks[0].position, chunks[1].position, chunks[2].position
        )
    }

    pub fn reload_chunks(&mut self, offset: na::Vector2<i32>) {
        if offset.y == -1 {
            let chunk_coords = self.chunks[-1].position + na::Vector2::new(0, 1);
            self.chunks.rotate_left(1);
            self.chunks.replace_last(Arc::new(Chunk {
                position: chunk_coords,
                chunk_index: RENDER_DISTANCE - 1,
                atlas_material_index: self.atlas_material_index,
            }));
        }
        // if offset.y == 1 {
        //     let chunk_coords = self.chunks.read().unwrap()[0].position + na::Vector2::new(0, -1);
        //     self.chunks.clone().write().unwrap().rotate_right(1);
        //     self.chunks.clone().write().unwrap().replace_first(Chunk {
        //         position: chunk_coords,
        //         chunk_index: RENDER_DISTANCE - 1,
        //         atlas_material_index: self.atlas_material_index,
        //     })
        // }
        self.center_chunk_position += offset;
    }

    pub fn render_chunks(
        chunks: Vec<Arc<Chunk>>,
        device: Arc<wgpu::Device>,
        mesh_manager: Arc<Mutex<MeshManager>>,
    ) -> Arc<Mutex<Vec<Option<Arc<Mesh>>>>> {
        let barrier = Arc::new(Barrier::new(chunks.len() + 1));
        let meshes: Arc<Mutex<Vec<Option<Arc<Mesh>>>>> =
            Arc::new(Mutex::new(vec![None; chunks.len()]));
        for (i, chunk_data) in chunks.iter().enumerate() {
            let chunk = chunk_data.clone();
            let device = device.clone();
            let mesh_manager = mesh_manager.clone();
            let barrier = barrier.clone();
            let meshes_clone = meshes.clone();
            std::thread::spawn(move || {
                let mesh = chunk.create_mesh(device, mesh_manager);
                meshes_clone.clone().lock().unwrap()[i] = Some(mesh.clone());
                barrier.wait();
            });
        }
        barrier.clone().wait();
        // for chunk in meshes.lock().unwrap().iter() {
        //     println!(
        //         "Chunk pos: {}",
        //         chunk.clone().transform.transform_matrix[3][2]
        //     );
        // }
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
                let mesh_manager = renderer
                    .module_manager
                    .mesh_manager
                    .as_ref()
                    .unwrap()
                    .clone();
                self.reload_chunks(offset);
                let device_arc = renderer.device.clone();
                let chunks = vec![self.chunks[-1].clone()];
                let new_chunks =
                    ChunkLoader::render_chunks(chunks, device_arc, mesh_manager).clone();
                for chunk in new_chunks.lock().unwrap().iter() {
                    self.chunk_meshes.rotate_left(1);
                    self.chunk_meshes.replace_last(chunk.clone());
                }
                self.is_pipeline_updated = false;

                self.center_chunk_position = chunked_position;
            }
            if !self.is_pipeline_updated {
                let mesh_manager = renderer
                    .module_manager
                    .mesh_manager
                    .as_ref()
                    .unwrap()
                    .clone();

                let chunk_meshes = self.chunk_meshes[..].iter().map(|c| c.clone());
                let num_meshes = mesh_manager
                    .clone()
                    .lock()
                    .unwrap()
                    .diffuse_pipeline_models
                    .len() as i32;
                if num_meshes - RENDER_DISTANCE as i32 >= 0 {
                    for (i, mesh) in chunk_meshes.enumerate() {
                        // dbg!(mesh.transform.transform_matrix);
                        mesh_manager.clone().lock().unwrap().diffuse_pipeline_models
                            [i + self.position_in_mesh_array] = mesh.clone().unwrap();
                    }
                } else {
                    for mesh in chunk_meshes {
                        mesh_manager
                            .clone()
                            .lock()
                            .unwrap()
                            .diffuse_pipeline_models
                            .push(mesh.clone().unwrap());
                    }
                }
                self.is_pipeline_updated = true;
                // mesh_manager.clone().lock().unwrap().diffuse_pipeline_models[]
            }
        }
    }
}
