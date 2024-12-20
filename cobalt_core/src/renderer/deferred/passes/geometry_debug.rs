use crate::renderer::{
    deferred::{
        depth_buffer::DepthBuffer, exports::Material, g_buffers::GeometryBuffers,
        screen_quad::ScreenQuad,
    },
    render_pass::RenderPass,
    renderable::RenderableTrait,
    renderer::RendererError,
};
use cobalt_graphics::{
    context::Graphics, vertex::UvVertex, CreateBindGroup, HasBindGroupLayout, HasStableBindGroup,
    HasVertexBufferLayout,
};

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum GeometryPassDebugMode {
    Normals = 0,
    Albedo = 1,
    Position = 2,
    Metallic = 3,
    Roughness = 4,
    Depth = 5,
}

pub struct GeometryDebugPass {
    pub mode: Option<GeometryPassDebugMode>,
    pipeline: wgpu::RenderPipeline,
    screen_quad: ScreenQuad,
}

impl GeometryDebugPass {
    pub fn new(graphics: &Graphics) -> Self {
        let pipeline_layout =
            graphics
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Geometry Debug Pass Pipeline Layout"),
                    bind_group_layouts: &[
                        &u32::bind_group_layout(graphics, ()),
                        &GeometryBuffers::bind_group_layout(graphics, ()),
                        &DepthBuffer::bind_group_layout(graphics, ()),
                    ],
                    push_constant_ranges: &[],
                });

        let shader = graphics
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Geometry Debug Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("geometry_debug.wgsl").into()),
            });

        let pipeline = graphics
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Geometry Debug Pass Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[UvVertex::vertex_buffer_layout()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: graphics.output_color_format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                depth_stencil: None,
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                    unclipped_depth: false,
                },
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        Self {
            mode: None,
            pipeline,
            screen_quad: ScreenQuad::new(),
        }
    }
}

impl RenderPass<(&GeometryBuffers, &DepthBuffer)> for GeometryDebugPass {
    /// Only called when debug mode is not `None`.
    fn draw(
        &mut self,
        frame: &mut cobalt_graphics::frame::Frame,
        graphics: &cobalt_graphics::context::Graphics,
        _frame_data: &mut crate::renderer::FrameData<Material>,
        extra_data: (&GeometryBuffers, &DepthBuffer),
    ) -> Result<(), RendererError> {
        let swap_texture = frame
            .swap_texture()
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let encoder = frame.get_encoder();

        let mode_bind_group = (self.mode.unwrap() as u32).create_bind_group(&graphics);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Geometry Debug Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &swap_texture,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        if let None = self.mode {
            return Err(RendererError::RenderPassError(
                "No debug mode set.".to_string(),
            ));
        }

        // Bind debug mode
        render_pass.set_bind_group(0, &mode_bind_group, &[]);

        // Bind geometry buffers
        render_pass.set_bind_group(1, &extra_data.0.stable_bind_group(), &[]);

        // Bind depth buffer
        render_pass.set_bind_group(2, &extra_data.1.stable_bind_group(), &[]);

        render_pass.set_pipeline(&self.pipeline);

        self.screen_quad.render(graphics, &mut render_pass);

        Ok(())
    }

    fn resize_callback(
        &mut self,
        _graphics: &cobalt_graphics::context::Graphics,
        _size: (u32, u32),
    ) -> Result<(), RendererError> {
        Ok(())
    }
}
