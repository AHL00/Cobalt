use crate::{ecs::component::Component, internal::aabb::AABB, resource::Resource, transform::Transform};

use super::material::Material;

pub mod plane;
pub mod mesh;

/// Enum that represents all renderable objects.
/// It eliminates the need for a trait object since we already know all the possible types.
/// It also implements the RenderableTrait trait which just passes the calls along to the actual structs.
pub enum Renderable {
    Plane(plane::Plane),
    Mesh(mesh::Mesh),
}

impl Component for Renderable {}

/// A trait implemented by all renderable objects
/// It allows for easy rendering of all renderable objects without needing to know the actual type.
/// The renderable's AABB are in local space.
pub(super) trait RenderableTrait {
    /// Assume that the uniforms and shader are already set
    fn render(&self, render_pass: &mut wgpu::RenderPass);

    fn get_material<'a>(&'a self) -> &'a crate::resource::Resource<crate::renderer::material::Material>;
}

impl Renderable {
    pub(crate) fn render(&self, render_pass: &mut wgpu::RenderPass) {
        match self {
            Self::Plane(plane) => plane.render(render_pass),
            Self::Mesh(mesh) => mesh.render(render_pass),
        }
    }

    pub(crate) fn get_material<'a>(&'a self) -> &'a Resource<Material> {
        match self {
            Self::Plane(plane) => &plane.material,
            Self::Mesh(mesh) => &mesh.material,
        }
    }

    // NOTE: Just generating a new world space aabb every time is easier
    // than trying to update it every time the transform changes.
    // It simplifies the code and reduces the insane amount of synchronization
    // issues and bugs. It reduces efficiency a bit but I think it's negligible.
    /// Returns the world space AABB of the renderable object.
    /// Transform needs to be mutable because if it is dirtym the model matrix will be recalculated.
    pub fn world_space_aabb(&self, transform: &mut Transform) -> AABB {
        let local_aabb =  match self {
            Self::Plane(plane) => &plane.local_space_aabb,
            Self::Mesh(mesh) => &mesh.local_space_aabb,
        };

        let world_space_aabb = local_aabb.transform_by_mat(&transform.model_matrix());

        world_space_aabb
    }
}