use wgpu::TextureView;

use super::{proj_view::ProjView, renderable::Renderable, renderer::FramePrepError};
use crate::{
    components::transform::Transform,
    exports::{
        ecs::{query::Optional, Entity, World},
        types::resource::{Resource, ResourceTrait},
    },
};
use cobalt_assets::exports::{Asset, AssetTrait};

/// Holds the data required to render a renderable.
pub struct RenderData<'a, M: ResourceTrait> {
    pub renderable: &'a Renderable,
    pub transform: &'a mut Transform,
    pub entity: Entity,
    pub in_frustum: bool,
    pub material: Resource<M>,
}

/// Holds the data required to render a frame.
/// It also helps generate that data from a few inputs using the `generate` method.
/// Materials are sorted or at least grouped together. Should reduce material binding count.
pub struct FrameData<'a, M: ResourceTrait + Ord> {
    pub depth_view: Option<wgpu::TextureView>,
    pub proj_view: ProjView,
    pub camera_position: ultraviolet::Vec3,
    pub render_data_vec: Vec<RenderData<'a, M>>,
}

impl<'a, M: ResourceTrait + AssetTrait + Ord> FrameData<'a, M> {
    /// Generates a list of `RenderData` from the world. It also performs other processing
    /// such as frustum culling and sorting by material.
    pub fn generate(
        world: &'a mut World,
        depth_view: Option<TextureView>,
        proj_view: ProjView,
        camera_pos: ultraviolet::Vec3,
    ) -> Result<Self, FramePrepError> {
        let mut render_data_vec = Vec::new();

        let renderable_query = world
            .query_mut::<(
                Transform,
                Renderable,
                Optional<Resource<M>>,
                Optional<Asset<M>>,
            )>()
            .map_err(|_| FramePrepError::NoRenderables)?;

        renderable_query.map(|(ent, (transform, renderable, resource_material, asset_material))| {
            // TODO: Normal matrix being calculated every frame, is this necessary?
            transform.calculate_normal_matrix(proj_view.view());

            let render_data = RenderData {
                renderable,
                transform,
                entity: ent,
                in_frustum: true,
                // TODO: Is it faster to clone the `Resource` or take a reference to it?
                material: {
                    // NOTE: Resource components take precedence over Asset components
                    if let Some(resource) = resource_material {
                        #[cfg(debug_assertions)]
                        {
                            if asset_material.is_some() {
                                log_once::warn_once!("Entity {:?} has both a resource and an asset material. The resource takes precedence and will be used.", ent);
                            }
                        }

                        resource.clone()
                    } else if let Some(asset) = asset_material {
                        asset.clone().into()
                    } else {
                        return Err(FramePrepError::NoMaterial(ent))
                    }
                },
            };

            Ok(render_data)
        })
        // iterate until error found
        .try_for_each(|x| {
            render_data_vec.push(x?);
            Ok(())
        })?;

        #[cfg(feature = "debug_stats")]
        let pre_cull_count = render_data_vec.len();
        // TODO: Implement frustum culling

        // NOTE: Shadow mapping should be done before culling
        // Can the shadow map do its own culling?
        //

        // // Sort by material
        // // TODO: Instead of sorting, maybe just group
        render_data_vec.sort_unstable_by(|a, b| {
            // if let Either::Left(a) = &a.material {
            //     if let Either::Left(b) = &b.material {
            //         a.borrow().cmp(&b.borrow())
            //     } else {
            //         std::cmp::Ordering::Less
            //     }
            // } else {
            //     std::cmp::Ordering::Greater
            // }

            a.material.borrow().cmp(&b.material.borrow())
        });

        #[cfg(feature = "debug_stats")]
        {
            let culled_count = pre_cull_count - render_data_vec.len();
            crate::stats::Stats::global().set("Culled entities", culled_count.into(), false);
            crate::stats::Stats::global().set("Rendered entities", render_data_vec.len().into(), false);
        }

        Ok(Self {
            depth_view,
            proj_view,
            camera_position: camera_pos,
            render_data_vec,
        })
    }
}
