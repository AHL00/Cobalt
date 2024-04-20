use crate::{ecs::component::Component, internal::aabb::AABB, resource::Resource, transform::Transform};

use super::material::Material;

pub mod plane;
pub mod mesh;

pub enum Renderable {
    Plane(plane::Plane),
    Mesh(mesh::Mesh),
}

impl Component for Renderable {}

impl Renderable {
    pub(super) fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        match self {
            Self::Plane(plane) => plane.render(render_pass),
            Self::Mesh(mesh) => mesh.render(render_pass),
        }
    }

    pub(super) fn get_material<'a>(&'a self) -> &'a Resource<Material> {
        match self {
            Self::Plane(plane) => &plane.material,
            Self::Mesh(mesh) => &mesh.material,
        }
    }

    pub(super) fn update_aabb(&mut self, transform: &mut Transform) {
        match self {
            Self::Plane(plane) => plane.update_aabb(transform),
            Self::Mesh(mesh) => mesh.update_aabb(transform),
        }
    }

    pub fn get_aabb(&self) -> &AABB {
        match self {
            Self::Plane(plane) => plane.get_aabb(),
            Self::Mesh(mesh) => mesh.get_aabb(),
        }
    }
}

/// A trait implemented by all renderable objects
trait RenderableTrait {
    /// Assume that the uniforms and shader are already set
    fn render(&self, render_pass: &mut wgpu::RenderPass);

    /// This should be triggered by the renderer if transform is 
    /// dirty. This will update the AABB for the renderable.
    fn update_aabb(&mut self, transform: &mut Transform);

    fn get_aabb(&self) -> &crate::internal::aabb::AABB;
}