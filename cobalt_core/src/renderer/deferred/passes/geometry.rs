use ultraviolet::Mat4;

use crate::{
    exports::components::Transform,
    graphics::{
        context::Graphics, vertex::UvNormalVertex, CreateBindGroup, HasBindGroup,
        HasBindGroupLayout, HasVertexBufferLayout,
    },
    renderer::{
        deferred::{exports::Material, g_buffers::GeometryBuffers},
        proj_view::ProjView,
        render_pass::RenderPass,
        renderer::RendererError,
    },
};

/// This pass will clear the geometry buffers and draw the scene to them.
/// It also clears and writes to the depth buffer.
pub struct GeometryPass {
    pipeline: wgpu::RenderPipeline,
    pub g_buffers: GeometryBuffers,
}

impl GeometryPass {
    pub fn new(graphics: &Graphics, output_size: (u32, u32)) -> Self {
        let layout = graphics.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Geometry Pass Pipeline Layout"),
                bind_group_layouts: &[
                    &Transform::bind_group_layout(graphics, ()),
                    &ProjView::bind_group_layout(graphics, ()),
                    &Material::bind_group_layout(graphics, ()),
                ],
                push_constant_ranges: &[],
            },
        );

        let shader =
            graphics
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Geometry Shader"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("geometry.wgsl").into()),
                });

        let pipeline = graphics.device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Geometry Pass Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[UvNormalVertex::vertex_buffer_layout()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    // Output of the fragment shader is as follows:
                    // - Position: Rgba16Float
                    // - Normal: Rgba16Float
                    // - Albedo / Metallic: Rgba8UnormSrgb [A_red; A_green; A_blue; S_intensity]
                    // - Diffuse: Rgba8UnormSrgb
                    targets: &[
                        Some(wgpu::ColorTargetState {
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                            format: GeometryBuffers::POSITION_FORMAT,
                        }),
                        Some(wgpu::ColorTargetState {
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                            format: GeometryBuffers::NORMAL_FORMAT,
                        }),
                        Some(wgpu::ColorTargetState {
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                            format: GeometryBuffers::ALBEDO_FORMAT,
                        }),
                        Some(wgpu::ColorTargetState {
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                            format: GeometryBuffers::METALLIC_ROUGHNESS_FORMAT,
                        }),
                    ],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: graphics.output_depth_format.unwrap(),
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState {
                        front: wgpu::StencilFaceState::IGNORE,
                        back: wgpu::StencilFaceState::IGNORE,
                        read_mask: 0,
                        write_mask: 0,
                    },
                    bias: wgpu::DepthBiasState {
                        constant: 0,
                        slope_scale: 0.0,
                        clamp: 0.0,
                    },
                }),
                multiview: None,
                multisample: wgpu::MultisampleState::default(),
                cache: None,
            },
        );

        Self {
            pipeline,
            g_buffers: GeometryBuffers::generate(graphics, output_size),
        }
    }

    pub fn get_g_buffers(&self) -> &GeometryBuffers {
        &self.g_buffers
    }
}

impl RenderPass<()> for GeometryPass {
    fn draw(
        &mut self,
        frame: &mut crate::graphics::frame::Frame,
        graphics: &crate::graphics::context::Graphics,
        frame_data: &mut crate::renderer::FrameData<Material>,
        _extra_data: (),
    ) -> Result<(), RendererError> {
        let depth_view = frame_data
            .depth_view
            .as_ref()
            .ok_or(RendererError::RenderPassError(
                "No depth view found.".to_string(),
            ))?;

        // TODO: For optimisation, cache the render pass descriptor and update only when textures change.
        let descriptor = wgpu::RenderPassDescriptor {
            label: Some("Geometry Pass Descriptor"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    view: &self.g_buffers.position_view,
                    resolve_target: None,
                }),
                Some(wgpu::RenderPassColorAttachment {
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    view: &self.g_buffers.normal_view,
                    resolve_target: None,
                }),
                Some(wgpu::RenderPassColorAttachment {
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    view: &self.g_buffers.albedo_view,
                    resolve_target: None,
                }),
                Some(wgpu::RenderPassColorAttachment {
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    view: &self.g_buffers.metallic_roughness_view,
                    resolve_target: None,
                }),
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        };

        let encoder = frame.get_encoder();

        let proj_view_bind_group = frame_data.proj_view.create_bind_group(&graphics);

        let mut render_pass = encoder.begin_render_pass(&descriptor);

        // Bind shader pipeline
        render_pass.set_pipeline(&self.pipeline);

        // ProjView bind group
        render_pass.set_bind_group(1, &proj_view_bind_group, &[]);

        let mut last_material_id: Option<usize> = None;

        for render_data in &mut frame_data.render_data_vec {
            // These are okay being unsafe as the assets/resources are guaranteed to live as long as they aren't dropped, and they are not.
            let mut material = render_data
                .material
                .left()
                .map(|m| unsafe { m.borrow_unsafe() });

            if let None = material {
                material = render_data
                    .material
                    .right()
                    .map(|m| unsafe { m.borrow_unsafe() });
            }

            let material = material.unwrap();

            if last_material_id.is_some() {
                if last_material_id.unwrap() != *material.id() {
                    // Bind material if it's different from the last one
                    render_pass.set_bind_group(2, &material.bind_group(), &[]);

                    last_material_id = Some(*material.id());
                }
            } else {
                // This is the first time, so bind the material
                render_pass.set_bind_group(2, &material.bind_group(), &[]);
            }

            // Transform bind group
            render_pass.set_bind_group(0, &render_data.transform.bind_group(graphics), &[]);

            // Let the renderable handle the drawing
            render_data.renderable.render(graphics, &mut render_pass);
        }

        Ok(())
    }

    fn resize_callback(&mut self, graphics: &Graphics, size: (u32, u32)) -> Result<(), RendererError> {
        self.g_buffers = GeometryBuffers::generate(graphics, size);
        Ok(())
    }
}
