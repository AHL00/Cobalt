use crate::{assets::Asset, renderer::{material::Material, mesh::MeshAsset}, resource::Resource};

use super::RenderableTrait;

pub struct Mesh {
    pub material: Resource<Material>,
    pub mesh: Asset<MeshAsset>,
}

impl Mesh {
    pub fn new(mesh: Asset<MeshAsset>, material: Resource<Material>) -> Self {
        Self { mesh, material }
    }
}

impl RenderableTrait for Mesh {
    fn render(&self, render_pass: &mut wgpu::RenderPass) {
        let mesh_asset = unsafe { self.mesh.borrow_unsafe() };

        render_pass.set_vertex_buffer(0, mesh_asset.vertex_buffer.slice(..));
        render_pass.set_index_buffer(mesh_asset.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..mesh_asset.num_indices, 0, 0..1);
    }
}