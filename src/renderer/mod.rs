use std::error::Error;

use ultraviolet::Mat4;

use crate::{
    ecs::{Entity, World}, engine::graphics, graphics::{CreateBindGroup, Frame, HasBindGroup}, stats::Stats, transform::Transform
};

use self::{camera::Camera, material::MaterialTrait, renderable::Renderable, proj_view::ProjView};

pub mod camera;
pub mod material;
pub mod mesh;
pub mod renderable;

mod proj_view;

pub trait Renderer {
    fn render(&mut self, frame: &mut Frame, world: &mut World, stats: &mut Stats);

    fn resize_callback(&mut self, size: (u32, u32)) -> Result<(), Box<dyn Error>>;
}

// TODO: Error handling
/// The RenderPass should not alter FrameData.
/// They are only mutable to allow for bind groups to be updated.
trait RenderPass {
    fn render(
        &mut self,
        frame: &mut Frame,
        proj_view: ProjView,
        frame_data: &mut FrameData,
    );

    fn create_wgpu_render_pass<'a>(
        &self,
        encoder: &'a mut wgpu::CommandEncoder,
        swap_texture: &'a wgpu::TextureView,
        frame_data: &'a wgpu::TextureView,
    ) -> wgpu::RenderPass<'a>;

    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

struct RenderData<'a> {
    renderable: &'a Renderable,
    transform: &'a mut Transform,
    entity: Entity,
    in_frustum: bool,
}

struct FrameData<'a> {
    depth_view: Option<wgpu::TextureView>,
    render_data_vec: Vec<RenderData<'a>>,
}

// TODO: Resize on window resize
pub struct DefaultRenderer {
    depth_texture: Option<wgpu::Texture>,
    forward_pass: ForwardPass,
}

struct ForwardPass {
    last_material_id: Option<u32>,
}

impl ForwardPass {
    fn new() -> Self {
        Self {
            last_material_id: None,
        }
    }
}

impl RenderPass for ForwardPass {
    fn render(
        &mut self,
        frame: &mut Frame,
        proj_view: ProjView,
        frame_data: &mut FrameData,
    ) {
        let swap_texture = frame.swap_texture().texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = frame.encoder();

        let proj_view_bind_group = proj_view.create_bind_group(&graphics().device);

        let mut render_pass = self.create_wgpu_render_pass(&mut encoder, &swap_texture, &frame_data.depth_view.as_ref().unwrap());

        render_pass.set_bind_group(1, &proj_view_bind_group, &[]);

        for render_data in &mut frame_data.render_data_vec {
            let material_resource = render_data.renderable.get_material();

            if self.last_material_id != Some(material_resource.id) {
                // On material change, set the new pipeline
                render_pass.set_pipeline(material_resource.borrow().get_pipeline());
                self.last_material_id = Some(material_resource.id);
            }

            // Set transform uniform
            render_pass.set_bind_group(0, &mut render_data.transform.bind_group(), &[]);

            // Set material bind group
            // Should be safe because the material is guaranteed to be the same for the entire frame
            // as the global RwLock graphics state is borrowed by renderer.
            unsafe { material_resource.borrow_unsafe().set_uniforms(1, &mut render_pass) };

            render_data.renderable.draw(&mut render_pass);
        }

        self.last_material_id = None;
    }

    fn create_wgpu_render_pass<'a>(
        &self,
        encoder: &'a mut wgpu::CommandEncoder,
        swap_texture: &'a wgpu::TextureView,
        depth_texture: &'a wgpu::TextureView,
    ) -> wgpu::RenderPass<'a> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Forward Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: swap_texture,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_texture,
                depth_ops: Some(wgpu::Operations {
                    // This is the first time the depth texture is used, it is cleared.
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        })
    }
}

impl Renderer for DefaultRenderer {
    fn render(&mut self, frame: &mut Frame, world: &mut World, stats: &mut Stats) {
        let camera_entity = self.get_camera(world);

        if let Some(camera_entity) = camera_entity {
            let cam_transform = world.get_component::<Transform>(camera_entity).unwrap();

            let view_matrix = Mat4::look_at(
                cam_transform.position(),
                cam_transform.position() + cam_transform.forward(),
                cam_transform.up(),
            );

            let proj_matrix = world
                .get_component_mut::<Camera>(camera_entity)
                .unwrap()
                .projection_matrix();

            let proj_view = ProjView::new(view_matrix, proj_matrix);

            self.clear_depth_texture(frame.encoder());

            // TODO: Retain this vec instead of creating it every frame
            let mut render_data_vec = Vec::new();

            let renderable_query = world.query_mut::<(Transform, Renderable)>().unwrap();

            for (ent, (transform, renderable)) in renderable_query {
                // Frustum test
                // TODO: Frustum culling, maybe expose this to the user?
                
                let render_data = RenderData {
                    renderable,
                    transform: transform,
                    entity: ent,
                    in_frustum: true,
                };

                render_data_vec.push(render_data);
            }

            // Sort by material
            // TODO: Instead of sorting, maybe just group
            render_data_vec.sort_unstable_by(|a, b| {
                a.renderable
                    .get_material()
                    .id
                    .cmp(&b.renderable.get_material().id)
            });

            // NOTE: Shadow mapping should be done before culling
            // Can the shadow map do its own culling?
            // 

            let pre_cull_count = render_data_vec.len();

            // TODO: Implement frustum culling


            stats.culled_entities = pre_cull_count - render_data_vec.len();
            stats.rendered_entities = render_data_vec.len();

            let mut frame_data = FrameData {
                depth_view: Some(self.depth_texture.as_ref().unwrap().create_view(
                    &wgpu::TextureViewDescriptor {
                        label: Some("Depth texture"),
                        format: Some(graphics().output_depth_format.expect("No depth format")),
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        aspect: wgpu::TextureAspect::DepthOnly,
                        base_mip_level: 0,
                        base_array_layer: 0,
                        array_layer_count: None,
                        mip_level_count: None,
                    },
                )),
                render_data_vec,
            };

            self.forward_pass.render(frame, proj_view, &mut frame_data);

        } else {
            log_once::warn_once!("No enabled camera found in scene.");
        }
    }

    fn resize_callback(&mut self, size: (u32, u32)) -> Result<(), Box<dyn Error>> {
        self.create_depth_texture(size)?;
        Ok(())
    }
}

impl DefaultRenderer {
    pub fn new() -> Self {
        Self {
            depth_texture: None,
            forward_pass: ForwardPass::new(),
        }
    }

    fn get_camera(&self, world: &World) -> Option<Entity> {
        let cam_query = world.query::<Camera>().unwrap();
        let mut enabled_camera_count = 0;
        let mut camera_entity = None;

        for (ent, cam) in cam_query {
            if cam.enabled {
                enabled_camera_count += 1;
            }

            // Make sure there is only one camera.
            if enabled_camera_count > 1 {
                log_once::warn_once!("More than one enabled camera entity found.");
                break;
            }

            // Make sure it has a transform.
            if let Some(_) = world.get_component::<Transform>(ent) {
                if cam.enabled {
                    camera_entity = Some(ent);
                }
                break;
            }

            log_once::warn_once!("Camera [{:?}] does not have a transform component.", ent);
        }

        camera_entity
    }

    fn create_depth_texture(&mut self, size: (u32, u32)) -> Result<(), Box<dyn Error>> {
        let graphics = crate::engine::graphics();

        let depth_texture = graphics.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: crate::engine::graphics()
                .output_depth_format
                .expect("No depth format"),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        self.depth_texture = Some(depth_texture);

        Ok(())
    }

    fn clear_depth_texture(&self, encoder: &mut wgpu::CommandEncoder) {
        if let Some(depth_texture) = &self.depth_texture {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }
    }
}
