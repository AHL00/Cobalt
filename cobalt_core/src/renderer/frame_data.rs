#![allow(dead_code)]

use wgpu::TextureView;

use super::{proj_view::ProjView, renderable::Renderable, renderer::FramePrepError};
use crate::{
    components::transform::Transform, ecs::entity::Entity, exports::ecs::World, stats::Stats,
};

/// Holds the data required to render a renderable.
pub struct RenderData<'a> {
    pub renderable: &'a Renderable,
    pub transform: &'a mut Transform,
    pub entity: Entity,
    pub in_frustum: bool,
}

/// Holds the data required to render a frame.
/// It also helps generate that data from a few inputs using the `generate` method.
pub struct FrameData<'a> {
    pub depth_view: Option<wgpu::TextureView>,
    pub proj_view: ProjView,
    pub render_data_vec: Vec<RenderData<'a>>,
}

impl<'a> FrameData<'a> {
    /// Generates a list of `RenderData` from the world. It also performs other processing
    /// such as frustum culling and sorting by material.
    pub fn generate(
        world: &'a mut World,
        depth_view: Option<TextureView>,
        proj_view: ProjView,
    ) -> Result<Self, FramePrepError> {
        let mut render_data_vec = Vec::new();

        let renderable_query = world
            .query_mut::<(Transform, Renderable)>()
            .map_err(|_| FramePrepError::NoRenderables)?;

        for (ent, (transform, renderable)) in renderable_query {
            let render_data = RenderData {
                renderable,
                transform,
                entity: ent,
                in_frustum: true,
            };

            render_data_vec.push(render_data);
        }

        #[cfg(feature = "debug_stats")]
        let pre_cull_count = render_data_vec.len();
        // TODO: Implement frustum culling

        // NOTE: Shadow mapping should be done before culling
        // Can the shadow map do its own culling?
        //

        // Sort by material
        // TODO: Instead of sorting, maybe just group
        render_data_vec.sort_unstable_by(|a, b| {
            a.renderable
                .get_material()
                .id
                .cmp(&b.renderable.get_material().id)
        });

        #[cfg(feature = "debug_stats")]
        {
            let culled_count = pre_cull_count - render_data_vec.len();
            Stats::global().set("Culled entities", culled_count.into(), false);
            Stats::global().set("Rendered entities", render_data_vec.len().into(), false);
        }

        Ok(Self {
            depth_view,
            proj_view,
            render_data_vec,
        })
    }
}
