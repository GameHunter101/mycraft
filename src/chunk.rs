use gamezap::model::{Mesh, MeshTransform, Vertex};
use nalgebra as na;
use wgpu::util::DeviceExt;

use crate::{
    cube::{BlockWrapper, Cube},
    utils::MeshTools,
};
pub struct Chunk {
    pub position: na::Vector3<f32>,
    pub blocks: Box<[[[u16; 16]; 16]; 256]>,
}

impl MeshTools for Chunk {
    fn create_mesh(
        &self,
        device: &wgpu::Device,
        mut mesh_manager: std::cell::RefMut<gamezap::model::MeshManager>,
    ) {
        let mut vertices = vec![];
        let mut indices = vec![];
        for (y, slice) in self.blocks.iter().enumerate() {
            for (x, row) in slice.iter().enumerate() {
                for (z, block) in row.iter().enumerate() {
                    // println!("{}, {}, {}", x, y, z,);
                    let block = Cube::new(
                        "cube",
                        self.position + na::Vector3::new(x as f32, y as f32, z as f32),
                        0,
                        BlockWrapper[*block as u32],
                    );
                    let mut recalculated_vertices = block
                        .vertices
                        .map(|v| {
                            let old_position = na::Vector3::from_column_slice(&v.position);
                            let new_position: [f32; 3] = (block.position + old_position).into();

                            Vertex {
                                position: new_position,
                                tex_coords: v.tex_coords,
                                normal: v.normal,
                                bitangent: v.bitangent,
                                tangent: v.tangent,
                            }
                        })
                        .to_vec();
                    let mut recalculated_indices = block
                        .indices
                        .map(|i| i + vertices.len() as u32)
                        .to_vec();
                    vertices.append(&mut recalculated_vertices);
                    indices.append(&mut recalculated_indices);
                }
            }
        }
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk 1 vert buff"),
            usage: wgpu::BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(&vertices),
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk 1 index buff"),
            usage: wgpu::BufferUsages::INDEX,
            contents: bytemuck::cast_slice(&indices),
        });

        let mesh = Mesh::new(
            device,
            "Chunk 1".to_string(),
            vertex_buffer,
            index_buffer,
            indices.len() as u32,
            MeshTransform::new(
                na::Vector3::new(0.0, 0.0, 0.0),
                na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), 0.0),
            ),
            0,
        );
        mesh_manager.diffuse_pipeline_models.push(mesh);
    }
}
