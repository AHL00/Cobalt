use crate::{
    exports::components::Transform,
    graphics::{
        context::Graphics, vertex::UvNormalVertex, CreateBindGroup, HasBindGroup, HasBindGroupLayout, HasVertexBufferLayout
    },
    renderer::{
        deferred::g_buffers::GeometryBuffers, proj_view::ProjView, render_pass::RenderPass,
    },
};

/// This pass will clear the geometry buffers and draw the scene to them.
/// It also clears and writes to the depth buffer.
pub struct GeometryPass {
    pub pipeline: wgpu::RenderPipeline,
    pub g_buffers: GeometryBuffers,
}

impl GeometryPass {
    pub fn new(output_size: (u32, u32)) -> Self {
        let layout = Graphics::global_read().device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Geometry Pass Pipeline Layout"),
                bind_group_layouts: &[
                    &Transform::bind_group_layout(),
                    &ProjView::bind_group_layout(),
                ],
                push_constant_ranges: &[],
            },
        );

        let shader =
            Graphics::global_read()
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Geometry Shader"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("geometry.wgsl").into()),
                });

        let pipeline = Graphics::global_read().device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Geometry Pass Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[UvNormalVertex::vertex_buffer_layout()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    // Output of the fragment shader is as follows:
                    // - Position: Rgba32Float
                    // - Normal: Rgba32Float
                    // - Albedo / Specular: Rgba8UnormSrgb [A_red; A_green; A_blue; S_intensity]
                    targets: &[
                        Some(wgpu::ColorTargetState {
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                            format: GeometryBuffers::POSITION_FORMAT,
                        }),
                        Some(wgpu::ColorTargetState {
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                            format: GeometryBuffers::NORMAL_FORMAT,
                        }),
                        Some(wgpu::ColorTargetState {
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                            format: GeometryBuffers::ALBEDO_SPECULAR_FORMAT,
                        }),
                    ],
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
                    format: Graphics::global_read().output_depth_format.unwrap(),
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
            },
        );

        Self {
            pipeline,
            g_buffers: GeometryBuffers::generate(output_size),
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
        proj_view: &crate::renderer::proj_view::ProjView,
        frame_data: &mut crate::renderer::FrameData,
        _extra_data: (),
    ) -> Result<(), Box<dyn std::error::Error>> {
        let depth_view = frame_data
            .depth_view
            .as_ref()
            .ok_or(String::from("Depth view not found"))?;

        // TODO: For optimisation, cache the render pass descriptor and update only when textures change.
        let descriptor = wgpu::RenderPassDescriptor {
            label: Some("Geometry Pass"),
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
                    view: &self.g_buffers.albedo_specular_view,
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

        let encoder = frame.encoder();

        let proj_view_bind_group = proj_view.create_bind_group(&graphics.device);
        
        let mut render_pass = encoder.begin_render_pass(&descriptor);

        // Bind shader pipeline
        render_pass.set_pipeline(&self.pipeline);

        // ProjView bind group
        render_pass.set_bind_group(1, &proj_view_bind_group, &[]);

        for render_data in &mut frame_data.render_data_vec {
            // Transform bind group
            render_pass.set_bind_group(0, &render_data.transform.bind_group(graphics), &[]);
            
            // Let the renderable handle the drawing
            render_data.renderable.render(&mut render_pass);
        }

        Ok(())
    }

    fn resize_callback(&mut self, size: (u32, u32)) -> Result<(), Box<dyn std::error::Error>> {
        self.g_buffers = GeometryBuffers::generate(size);
        Ok(())
    }
}
