use std::{any::Any, error::Error, sync::LazyLock};

use ultraviolet::Mat4;
use wgpu::util::DeviceExt;

use crate::{
    ecs::World,
    engine::graphics,
    graphics::{CreateBindGroup, Frame, HasBindGroupLayout},
    transform::Transform,
};

use self::camera::Camera;

pub mod camera;
pub mod sprite;
pub mod mesh;
pub mod material;
mod default_pipelines;

pub(crate) struct ViewProj {
    view: ultraviolet::Mat4,
    proj: ultraviolet::Mat4,
}

static VIEW_PROJ_BIND_GROUP_LAYOUT: LazyLock<wgpu::BindGroupLayout> = LazyLock::new(|| {
    graphics()
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
});

impl HasBindGroupLayout for ViewProj {
    fn bind_group_layout() -> &'static wgpu::BindGroupLayout {
        &*VIEW_PROJ_BIND_GROUP_LAYOUT
    }
}

impl CreateBindGroup for ViewProj {
    fn create_bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup {
        let view_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(self.view.as_byte_slice()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let proj_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(self.proj.as_byte_slice()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &*VIEW_PROJ_BIND_GROUP_LAYOUT,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(view_buffer.as_entire_buffer_binding()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(proj_buffer.as_entire_buffer_binding()),
                },
            ],
        })
    }
}

pub(crate) struct RenderData {
    depth_view: Option<wgpu::TextureView>,
}

/// This trait is used to define a pipeline for the renderer.
/// It renders all components of a specific type in an ECS world.
pub(crate) trait RendererPipeline {
    fn render(
        &mut self,
        frame: &mut Frame,
        world: &mut World,
        view_proj: ViewProj,
        render_data: &RenderData,
    );

    fn create_wgpu_render_pass<'a>(
        &self,
        encoder: &'a mut wgpu::CommandEncoder,
        swap_texture: &'a wgpu::TextureView,
        depth_view: &'a RenderData,
    ) -> wgpu::RenderPass<'a>;

    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

pub struct Renderer {
    pipelines: Vec<Box<dyn RendererPipeline>>,
    depth_texture: Option<wgpu::Texture>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            pipelines: Vec::new(),
            depth_texture: None,
        }
    }

    pub(crate) fn resize_callback(&mut self, size: (u32, u32)) {
        self.create_depth_texture(size).unwrap();
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
            format: crate::engine::graphics().output_depth_format.expect("No depth format"),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        self.depth_texture = Some(depth_texture);

        Ok(())
    }

    pub(crate) fn add_pipeline<T: RendererPipeline + 'static>(&mut self, pipeline: T) {
        // Make sure pipeline doesn't already exist.
        for existing_pipeline in &self.pipelines {
            if std::any::TypeId::of::<T>() == existing_pipeline.type_id() {
                panic!("Pipeline already exists");
            }
        }

        self.pipelines.push(Box::new(pipeline));
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

    pub fn render(&mut self, frame: &mut Frame, world: &mut World) {
        // Get camera
        let cam_query = world.query::<Camera>().unwrap();

        let mut camera_entity = None;
        let mut enabled_camera_count = 0;

        {
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
        }

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

            self.clear_depth_texture(frame.encoder());

            let render_data = RenderData {
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
            };

            // Render
            for pipeline in &mut self.pipelines {
                pipeline.render(
                    frame,
                    world,
                    ViewProj {
                        view: view_matrix,
                        proj: proj_matrix,
                    },
                    &render_data,
                );
            }
        } else {
            log_once::warn_once!("No enabled camera found in scene.");
        }
    }
}
