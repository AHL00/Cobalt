use cobalt_assets::{asset::Asset, server::AssetServer};
use cobalt_ecs::exports::Component;
use cobalt_graphics::context::Graphics;
use serde::{Deserialize, Serialize};
use crate::{components::transform::Transform, types::{aabb::AABB, either::Either, resource::Resource}};

use super::deferred::exports::Material;

pub mod mesh;
pub mod plane;

/// Enum that represents all renderable objects.
/// It eliminates the need for a trait object since we already know all the possible types.
/// It also implements the RenderableTrait trait which just passes the calls along to the actual structs.
pub enum Renderable {
    Plane(plane::Plane),
    Mesh(mesh::MeshRenderable),
}

impl Component for Renderable {
    type DeContext<'a> = (&'a Graphics, &'a mut AssetServer);
    type SerContext<'a> = ();

    fn deserialise<'de, D>(context: Self::DeContext<'de>, deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        todo!()
    }

    fn serialize<'se, S>(&self, context: Self::SerContext<'se>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        todo!()
    }
}

/// A trait implemented by all renderable objects
/// It allows for easy rendering of all renderable objects without needing to know the actual type.
/// The renderable's AABB are in local space.
pub(super) trait RenderableTrait {
    /// Assume that the uniforms and shader are already set
    fn render(&self, graphics: &Graphics, render_pass: &mut wgpu::RenderPass);
}

impl Renderable {
    pub(crate) fn render(&self, graphics: &Graphics, render_pass: &mut wgpu::RenderPass) {
        match self {
            Self::Plane(plane) => plane.render(graphics, render_pass),
            Self::Mesh(mesh) => mesh.render(graphics, render_pass),
        }
    }

    // NOTE: Just generating a new world space aabb every time is easier
    // than trying to store and update it every time the transform changes.
    // It simplifies the code and reduces the insane amount of synchronization
    // issues and bugs. It reduces efficiency a bit but I think it's negligible.
    /// Returns the world space AABB of the renderable object.
    /// Transform needs to be mutable because if it is dirtym the model matrix will be recalculated.
    pub fn world_space_aabb(&self, transform: &mut Transform) -> AABB {
        let local_aabb = match self {
            Self::Plane(plane) => &plane.local_space_aabb,
            Self::Mesh(mesh) => &mesh.local_space_aabb,
        };

        let world_space_aabb = local_aabb.multiply_by_matrix(&transform.model_matrix());

        world_space_aabb
    }
}
