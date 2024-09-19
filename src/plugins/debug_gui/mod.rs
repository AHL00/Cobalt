use cobalt_core::graphics::{context::Graphics, wgpu, window::WindowInternal};
use cobalt_runtime::{app::App, plugins::Plugin};
use egui_winit::EventResponse;

mod windows;

pub mod exports {
    pub use super::windows::DebugMenu;
    pub use egui;
}

// TODO: Add prebuilt debug windows such as StatsWindow.

/// Needs to run at a higher priority than other plugins that consume events if GUI input is to not be blocked by other plugins.
/// If there are post-render plugins that render on top of the frame, they should run before this plugin to make sure the GUI is on top.
pub struct DebugGUIPlugin {
    ctx: Option<egui::Context>,
    renderer: Option<egui_wgpu::Renderer>,
    state: Option<egui_winit::State>,
    draw_ui: Box<dyn Fn(&egui::Context, &mut cobalt_runtime::engine::Engine, &mut dyn App)>,
    debug_menu: windows::DebugMenu,
    enabled: bool,
    pub show_built_in_menus: bool,
}

impl DebugGUIPlugin {
    // NOTE: This has to match the MSAA samples of the main renderer.
    // TODO: Make this read from the renderer's configuration.
    const MSAA_SAMPLES: u32 = 1;
}

impl Default for DebugGUIPlugin {
    fn default() -> Self {
        Self {
            ctx: None,
            renderer: None,
            state: None,
            draw_ui: Box::new(|_, _, _| {}),
            debug_menu: windows::DebugMenu::new(),
            enabled: true,
            show_built_in_menus: true,
        }
    }
}

impl DebugGUIPlugin {
    pub fn set_draw_ui(
        &mut self,
        draw_ui: impl Fn(&egui::Context, &mut cobalt_runtime::engine::Engine, &mut dyn App) + 'static,
    ) {
        self.draw_ui = Box::new(draw_ui);
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
}

impl Plugin for DebugGUIPlugin {
    fn name(&self) -> &'static str {
        "Debug GUI"
    }

    fn startup(
        &mut self,
        engine: &mut cobalt_runtime::engine::Engine,
        _app: &mut dyn App,
    ) -> Result<(), cobalt_runtime::plugins::plugin::PluginError> {
        log::info!("Initializing egui context...");

        self.ctx = Some(egui::Context::default());

        let ctx = self.ctx.as_ref().unwrap();

        let id = ctx.viewport_id();

        self.state = Some(egui_winit::State::new(
            ctx.clone(),
            id,
            engine.window().winit(),
            None,
            None,
            None,
        ));

        let graphics = engine.graphics();

        self.renderer = Some(egui_wgpu::Renderer::new(
            &graphics.device,
            graphics.output_color_format,
            None,
            DebugGUIPlugin::MSAA_SAMPLES,
            true
        ));

        Ok(())
    }

    fn event(
        &mut self,
        engine: &mut cobalt_runtime::engine::Engine,
        event: cobalt_core::reexports::winit::event::Event<()>,
        _app: &mut dyn App,
    ) -> Result<bool, cobalt_runtime::plugins::plugin::PluginError> {
        if !self.enabled {
            return Ok(false);
        }

        let mut event_consumed = false;

        match event {
            egui_winit::winit::event::Event::WindowEvent { event, window_id } => {
                if window_id == engine.window().winit().id() {
                    if let Some(state) = self.state.as_mut() {
                        let res = state.on_window_event(engine.window().winit(), &event);

                        match res {
                            EventResponse { consumed, repaint } => {
                                event_consumed = consumed;

                                if repaint {
                                    engine.window().winit().request_redraw();
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(event_consumed)
    }

    fn post_render(
        &mut self,
        engine: &mut cobalt_runtime::engine::Engine,
        frame: &mut cobalt_core::graphics::frame::Frame,
        app: &mut dyn App,
    ) -> Result<(), cobalt_runtime::plugins::plugin::PluginError> {
        if !self.enabled {
            return Ok(());
        }

        let ctx = self.ctx.as_ref().unwrap();
        let state = self.state.as_mut().unwrap();
        let renderer = self.renderer.as_mut().unwrap();

        let raw_input = state.take_egui_input(&engine.window().winit());

        ctx.begin_frame(raw_input);

        (self.draw_ui)(&ctx, engine, app);

        if self.show_built_in_menus {
            self.debug_menu.show(&ctx, engine, app);
        }

        let full_output = ctx.end_frame();

        state.handle_platform_output(&engine.window().winit(), full_output.platform_output);

        let tris = ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

        let graphics = engine.graphics();

        for (id, image_delta) in full_output.textures_delta.set {
            renderer.update_texture(&graphics.device, &graphics.queue, id, &image_delta);
        }

        let inner_size = engine.window().winit().inner_size();

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            pixels_per_point: full_output.pixels_per_point,
            size_in_pixels: [inner_size.width, inner_size.height],
        };

        renderer.update_buffers(
            &graphics.device,
            &graphics.queue,
            frame.get_encoder(),
            &tris,
            &screen_descriptor,
        );

        let view = frame
            .swap_texture()
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut render_pass = frame
            .get_encoder()
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
                label: Some("Egui Render Pass"),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

        renderer.render(&mut render_pass, &tris, &screen_descriptor);

        drop(render_pass);

        for texture_id in full_output.textures_delta.free {
            renderer.free_texture(&texture_id);
        }

        Ok(())
    }
}
