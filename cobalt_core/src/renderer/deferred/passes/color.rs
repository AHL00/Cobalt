use crate::renderer::{
    deferred::{
        depth_buffer::DepthBuffer,
        g_buffers::GeometryBuffers,
        screen_quad::{ScreenQuad, ScreenQuadVertexFormat},
    },
    render_pass::RenderPass,
    renderable::RenderableTrait,
    renderer::RendererError,
};

use cobalt_graphics::{
    context::Graphics, CreateBindGroup, HasBindGroupLayout, HasStableBindGroup,
    HasVertexBufferLayout,
};

pub struct ColorPass {
    pipeline: wgpu::RenderPipeline,
    screen_quad: ScreenQuad,
}

impl ColorPass {
    pub fn new(graphics: &Graphics, _output_size: (u32, u32)) -> Self {
        let layout = graphics
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Color Pass Pipeline Layout"),
                bind_group_layouts: &[
                    &GeometryBuffers::bind_group_layout(graphics, ()),
                    &DepthBuffer::bind_group_layout(graphics, ()),
                    &ultraviolet::Vec3::bind_group_layout(graphics, ()),
                ],
                push_constant_ranges: &[],
            });

        let shader = graphics
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Color Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("color.wgsl").into()),
            });

        let pipeline = graphics
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Color Pass Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[ScreenQuadVertexFormat::vertex_buffer_layout()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                        format: graphics.output_color_format,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multiview: None,
                multisample: wgpu::MultisampleState::default(),
                cache: None,
            });

        Self {
            pipeline,
            screen_quad: ScreenQuad::new(),
        }
    }
}

pub struct ColorPassInput<'a> {
    pub geometry_buffers: &'a GeometryBuffers,
    pub depth_buffer: &'a DepthBuffer,
    pub cam_position: ultraviolet::Vec3,
}

impl<'a> RenderPass<ColorPassInput<'a>> for ColorPass {
    fn draw(
        &mut self,
        frame: &mut cobalt_graphics::frame::Frame,
        graphics: &cobalt_graphics::context::Graphics,
        _frame_data: &mut crate::renderer::FrameData<super::super::Material>,
        extra_data: ColorPassInput<'a>,
    ) -> Result<(), crate::renderer::renderer::RendererError> {
        let swap_view = &frame
            .swap_texture()
            .texture
            .create_view(&Default::default());

        let descriptor = wgpu::RenderPassDescriptor {
            label: Some("Color Pass Descriptor"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                ops: wgpu::Operations {
                    // Clear the screen to black
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                view: swap_view,
                resolve_target: None,
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        };

        let encoder = frame.get_encoder();

        let cam_pos_bind_group = extra_data.cam_position.create_bind_group(&graphics);

        let mut render_pass = encoder.begin_render_pass(&descriptor);

        // Bind geometry buffers
        render_pass.set_bind_group(0, &extra_data.geometry_buffers.stable_bind_group(), &[]);

        // Bind depth buffer
        render_pass.set_bind_group(1, &extra_data.depth_buffer.stable_bind_group(), &[]);

        // Bind camera position
        render_pass.set_bind_group(2, &cam_pos_bind_group, &[]);

        render_pass.set_pipeline(&self.pipeline);

        self.screen_quad.render(graphics, &mut render_pass);

        Ok(())
    }

    fn resize_callback(
        &mut self,
        _graphics: &Graphics,
        _size: (u32, u32),
    ) -> Result<(), RendererError> {
        Ok(())
    }
}
