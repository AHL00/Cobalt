use crate::{ecs::component::Component, resource::Resource};

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
}

/// A trait implemented by all renderable objects
trait RenderableTrait {
    /// Assume that the uniforms and shader are already set
    fn render(&self, render_pass: &mut wgpu::RenderPass);
}