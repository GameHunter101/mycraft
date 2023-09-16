use std::{
    cell::RefMut,
    sync::{Arc, Mutex},
};

use gamezap::model::MeshManager;

pub trait MeshTools {
    fn create_mesh(&self, device: &wgpu::Device, mesh_manager: Arc<Mutex<MeshManager>>);
}
