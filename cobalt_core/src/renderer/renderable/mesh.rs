use crate::{renderer::mesh::MeshAsset, types::aabb::AABB};
use cobalt_assets::{self, asset::Asset};
use cobalt_graphics::context::Graphics;

use super::RenderableTrait;

pub struct Mesh {
    pub mesh: Asset<MeshAsset>,
    pub(crate) local_space_aabb: AABB,
}

impl Mesh {
    pub fn new(mesh: Asset<MeshAsset>) -> Self {
        let local_space_aabb = mesh.borrow().local_aabb.clone();

        Self {
            mesh,
            local_space_aabb,
        }
    }
}

impl RenderableTrait for Mesh {
    fn render(&self, graphics: &Graphics, render_pass: &mut wgpu::RenderPass) {
        let mesh_asset = unsafe { self.mesh.borrow_unsafe() };

        render_pass.set_vertex_buffer(0, mesh_asset.vertex_buffer.slice(..));
        render_pass.set_index_buffer(mesh_asset.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..mesh_asset.num_indices, 0, 0..1);
    }
}
