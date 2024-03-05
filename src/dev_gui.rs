pub use egui;
use egui::Visuals;
use egui_wgpu::ScreenDescriptor;
use winit::{event::WindowEvent, window::Window};

use crate::engine::{graphics, DynApp};

pub struct DevGui {
    pub(crate) ctx: egui::Context,
    pub(crate) renderer: egui_wgpu::Renderer,
    pub(crate) state: egui_winit::State,

    /// The function to run the UI.
    run_ui: Box<dyn Fn(&egui::Context, &mut crate::engine::Engine, &mut DynApp)>,
}

impl DevGui {
    pub(crate) fn new(
        device: &wgpu::Device,
        output_color_format: wgpu::TextureFormat,
        msaa_samples: u32,
        window: &Window,
    ) -> Self {
        log::info!("Initializing egui");

        let ctx = egui::Context::default();
        let id = ctx.viewport_id();

        let state = egui_winit::State::new(ctx.clone(), id, &window, None, None);

        // egui_state.set_pixels_per_point(window.scale_factor() as f32);
        let renderer = egui_wgpu::Renderer::new(device, output_color_format, None, msaa_samples);

        Self {
            ctx,
            renderer,
            state,
            run_ui: Box::new(|_, _, _| {}),
        }
    }

    /// Set the EGUI visuals.
    /// Refer to egui documentation for more information.
    pub fn set_visuals(&mut self, visuals: Visuals) {
        self.ctx.set_visuals(visuals);
    }

    pub fn set_ui(&mut self, run: impl Fn(&egui::Context, &mut crate::engine::Engine, &mut DynApp) + 'static) {
        self.run_ui = Box::new(run);
    }

    /// Handle a winit window event. This should be called in your winit event loop.
    /// Returns true if the event was used by egui and should not be used for anything else.
    pub(crate) fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        let response = self.state.on_window_event(window, event);

        response.consumed
    }

    pub(crate) fn render(
        &mut self,
        engine: &mut crate::engine::Engine,
        app: &mut DynApp,
        frame: &mut crate::graphics::Frame,
        screen_descriptor: &ScreenDescriptor,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let raw_input = self.state.take_egui_input(&engine.window.winit);

        self.ctx.begin_frame(raw_input);

        (self.run_ui)(&self.ctx, engine, app);

        let full_output = self.ctx.end_frame();

        self.state
            .handle_platform_output(&engine.window.winit, full_output.platform_output);

        let tris = self
            .ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        for (id, image_delta) in full_output.textures_delta.set {
            self.renderer
                .update_texture(&graphics().device, &graphics().queue, id, &image_delta);
        }

        self.renderer.update_buffers(
            &graphics().device,
            &graphics().queue,
            frame.encoder(),
            &tris,
            &screen_descriptor,
        );

        let view = frame
            .swap_texture()
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut render_pass = frame
            .encoder()
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                label: Some("Egui render pass"),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

        self.renderer
            .render(&mut render_pass, &tris, &screen_descriptor);

        drop(render_pass);

        for texture_id in full_output.textures_delta.free {
            self.renderer.free_texture(&texture_id);
        }

        Ok(())
    }
}
