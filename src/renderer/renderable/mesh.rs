use crate::{
    assets::Asset,
    internal::aabb::AABB,
    renderer::{material::Material, mesh::MeshAsset},
    resource::Resource,
    transform::Transform,
};

use super::RenderableTrait;

pub struct Mesh {
    pub material: Resource<Material>,
    pub mesh: Asset<MeshAsset>,
    /// The axis-aligned bounding box for the mesh
    /// It is updated every time the mesh is transformed
    /// This update should be triggered by the renderer,
    /// before the mesh is drawn
    world_space_aabb: AABB,
}

impl Mesh {
    pub fn new(mesh: Asset<MeshAsset>, material: Resource<Material>) -> Self {
        Self { mesh, material, world_space_aabb: AABB::zero() }
    }
}

impl RenderableTrait for Mesh {
    fn render(&self, render_pass: &mut wgpu::RenderPass) {
        let mesh_asset = unsafe { self.mesh.borrow_unsafe() };

        render_pass.set_vertex_buffer(0, mesh_asset.vertex_buffer.slice(..));
        render_pass.set_index_buffer(mesh_asset.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..mesh_asset.num_indices, 0, 0..1);
    }

    fn get_aabb(&self) -> &AABB {
        &self.world_space_aabb
    }

    fn update_aabb(&mut self, transform: &mut Transform) {
        self.world_space_aabb = self.mesh.borrow().local_aabb.transform(transform.model_matrix());
    }
}
