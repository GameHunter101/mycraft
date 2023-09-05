use std::cell::RefMut;

use gamezap::model::MeshManager;

pub trait MeshTools {
    fn create_mesh(&self, device: &wgpu::Device, mesh_manager: RefMut<MeshManager>);
}
