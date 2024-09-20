pub struct Frame {
    pub(super) encoder: wgpu::CommandEncoder,
    pub(super) swap_texture: wgpu::SurfaceTexture,
}

impl Frame {
    /// TODO: For multithreading, we could make new encoders every time this is called.
    /// Store the encoders in a vec and then submit them all at the end of the frame.
    pub fn get_encoder(&mut self) -> &mut wgpu::CommandEncoder {
        &mut self.encoder
    }

    pub fn swap_texture(&self) -> &wgpu::SurfaceTexture {
        &self.swap_texture
    }

    pub fn clear(&mut self, color: wgpu::Color) {
        let view = self
            .swap_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let _render_pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            timestamp_writes: None,
            occlusion_query_set: None,
            depth_stencil_attachment: None,
        });
    }
}