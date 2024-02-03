use std::{any::Any, borrow::Cow, sync::LazyLock};

use serde::{Deserialize, Serialize};
use wgpu::util::DeviceExt;

use crate::{
    ecs::{component::Component, World},
    engine::graphics,
    graphics::{CreateBindGroup, Frame, Graphics, HasBindGroupLayout},
    transform::Transform,
};

use self::camera::Camera;

pub mod camera;
pub mod sprite;

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

/// This trait is used to define a pipeline for the renderer.
/// It renders all components of a specific type in an ECS world.
pub trait RendererPipeline {
    fn render(&mut self, frame: &mut Frame, world: &mut World, view_proj: ViewProj);

    fn create_wgpu_pipeline(&self, graphics: &Graphics) -> wgpu::RenderPipeline;

    fn create_wgpu_render_pass<'a>(
        &self,
        encoder: &'a mut wgpu::CommandEncoder,
        swap_texture: &'a wgpu::TextureView,
    ) -> wgpu::RenderPass<'a>;

    fn name(&self) -> &str;
}

pub struct Renderer {
    pipelines: Vec<Box<dyn RendererPipeline>>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            pipelines: Vec::new(),
        }
    }

    pub(crate) fn add_default_pipelines(&mut self) {
        let graphics = crate::engine::graphics();

        self.add_pipeline(sprite::SpritePipeline::new(&graphics));
    }

    pub fn add_pipeline<T: RendererPipeline + 'static>(&mut self, pipeline: T) {
        // Make sure pipeline doesn't already exist.
        for existing_pipeline in &self.pipelines {
            if std::any::TypeId::of::<T>() == existing_pipeline.type_id() {
                panic!("Pipeline already exists");
            }
        }

        self.pipelines.push(Box::new(pipeline));
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
                if let Some(transform) = world.get_component::<Transform>(ent) {
                    if cam.enabled {
                        camera_entity = Some(ent);
                    }
                    break;
                }

                log_once::warn_once!("Camera [{:?}] does not have a transform component.", ent);
            }
        }

        if let Some(camera_entity) = camera_entity {
            let view_matrix = world
                .get_component_mut::<Transform>(camera_entity)
                .unwrap()
                .model_matrix()
                .inversed();

            let proj_matrix = world
                .get_component_mut::<Camera>(camera_entity)
                .unwrap()
                .projection_matrix();

            // Render
            for pipeline in &mut self.pipelines {
                pipeline.render(
                    frame,
                    world,
                    ViewProj {
                        view: view_matrix,
                        proj: proj_matrix,
                    },
                );
            }
        } else {
            log_once::warn_once!("No enabled camera found in scene.");
        }
    }
}