use std::{error::Error, time::Duration};

use cobalt_core::{
    graphics::{
        context::Graphics, exports::wgpu, window::WindowInternal, winit::{
            self,
            event::{Event, WindowEvent},
        }
    },
    input::InputInternal,
    stats::{Stat, Stats, StatsInternal},
};

use crate::{
    app::App,
    plugins::{
        manager::PluginInternal,
        plugin::PluginError,
        PluginManager,
    },
};

use super::exports::Engine;

/// The main loop of the engine. This function will run the engine and the provided app.
pub fn run(
    mut engine: Engine,
    mut plugins: PluginManager,
    app: &mut dyn App,
) -> Result<(), Box<dyn Error>> {
    log::info!("Running engine...");

    let mut last_app_update = std::time::Instant::now();
    let mut last_avg_fps_update = std::time::Instant::now();
    let mut last_second_frames = 0;

    app.on_start(&mut engine, &mut plugins);

    for (plugin, _, _) in plugins.get_plugins_in_order() {
        let res = plugin.startup(&mut engine, app);

        if let Err(e) = res {
            match e {
                PluginError::Fatal(e) => {
                    log::error!(
                        "Plugin '{}' failed to start: {:?}. Fatal error, stopping...",
                        plugin.name(),
                        e
                    );
                    return Err(e);
                }
                PluginError::NonFatal(e) => {
                    log::error!(
                        "Plugin '{}' failed to start: {:?}. Non-fatal error, continuing...",
                        plugin.name(),
                        e
                    );
                    continue;
                }
            }
        }

        log::info!("Plugin '{}' started successfully.", plugin.name());
    }

    engine.event_loop.take().unwrap().run(move |event, elwt| {
        elwt.set_control_flow(winit::event_loop::ControlFlow::Poll);

        // If the engine has requested an exit, exit the event loop.
        // Then, the events will be exhausted and the engine will shutdown.
        // In the meantime, the plugins will not be updated.
        if engine.exit_requested {
            elwt.exit();
        } else {
            for (plugin, _, _) in plugins.get_plugins_in_order() {
                let res = plugin.update(&mut engine, app);
    
                if let Err(e) = res {
                    match e {
                        PluginError::Fatal(e) => {
                            log::error!(
                                "Plugin '{}' failed in update: {:?}. Fatal error, stopping...",
                                plugin.name(),
                                e
                            );
                            engine.exit();
                        }
                        PluginError::NonFatal(e) => {
                            log::error!(
                                "Plugin '{}' failed in update: {:?}. Non-fatal error, continuing...",
                                plugin.name(),
                                e
                            );
                            continue;
                        }
                    }
                }
            }
        }

        // TODO: If close is requested, maybe ignore every other event?

        // Let plugins process events before the engine.
        let mut plugin_consumed_event = false;

        for (plugin, _, _) in plugins.get_plugins_in_order() {
            let res = plugin.as_mut().event(&mut engine, event.clone(), app);

            if let Err(e) = res {
                match e {
                    PluginError::Fatal(e) => {
                        log::error!(
                            "Plugin '{}' failed in event: {:?}. Fatal error, stopping...",
                            plugin.name(),
                            e
                        );
                        engine.exit();
                    }
                    PluginError::NonFatal(e) => {
                        log::error!(
                            "Plugin '{}' failed in event: {:?}. Non-fatal error, continuing...",
                            plugin.name(),
                            e
                        );
                        continue;
                    }
                }
            } else {
                if res.unwrap() {
                    plugin_consumed_event = true;
                    break;
                }
            }
        }

        if plugin_consumed_event {
            return;
        }

        match event {
            Event::WindowEvent { event, window_id } if window_id == engine.window.winit().id() => {
                // If event was consumed, no need to keep matching.
                let (input_new_event, input_consumed_event) =  engine.input.update(&event);

                if let Some(event) = input_new_event {
                    // There are changes in the input
                    app.on_input(&mut engine, &mut plugins, event);
                }

                if input_consumed_event {
                    return;
                }

                match event {
                    WindowEvent::CloseRequested => {
                        app.on_stop(&mut engine, &mut plugins);

                        for (plugin, _, _) in plugins.get_plugins_in_order() {
                            let res = plugin.shutdown(&mut engine, app);

                            if let Err(e) = res {
                                match e {
                                    PluginError::Fatal(e) => {
                                        log::error!(
                                            "Plugin '{}' failed to shutdown: {:?}.",
                                            plugin.name(),
                                            e
                                        );
                                    }
                                    PluginError::NonFatal(e) => {
                                        log::error!(
                                            "Plugin '{}' failed to shutdown: {:?}.",
                                            plugin.name(),
                                            e
                                        );
                                    }
                                }
                            }

                            log::info!("Plugin '{}' shutdown successfully.", plugin.name());
                        }

                        elwt.exit()
                    }
                    WindowEvent::RedrawRequested => {
                        for (plugin, _, _) in plugins.get_plugins_in_order() {
                            let res = plugin.pre_render(&mut engine, app);

                            if let Err(e) = res {
                                match e {
                                    PluginError::Fatal(e) => {
                                        log::error!(
                                            "Plugin '{}' failed in pre_render: {:?}. Fatal error, stopping...",
                                            plugin.name(),
                                            e
                                        );
                                        engine.exit();
                                    }
                                    PluginError::NonFatal(e) => {
                                        log::error!(
                                            "Plugin '{}' failed in pre_render: {:?}. Non-fatal error, continuing...",
                                            plugin.name(),
                                            e
                                        );
                                        continue;
                                    }
                                }
                            }
                        }

                        {
                            let delta_time = last_app_update.elapsed().as_secs_f32();
                            app.on_update(&mut engine, &mut plugins, delta_time);
                            last_app_update = std::time::Instant::now();
                        }

                        let cpu_render_start = std::time::Instant::now();

                        let graphics = Graphics::global_read();

                        let mut frame = graphics.begin_frame().unwrap();

                        frame.clear(wgpu::Color::BLACK);

                        let prep_res = engine.renderer.prep_frame(&mut frame, &mut engine.scene.world);


                        let frame_data = match prep_res {
                            Ok(frame_data) => {
                                Some(frame_data)
                            }
                            Err(e) => {
                                match e {
                                    cobalt_core::renderer::renderer::FramePrepError::NoMaterial(_) => {
                                        log::error!("Fatal error during frame prep: {:?}", e);

                                        engine.exit();

                                        None
                                    }
                                    _ => {
                                        // Non-fatal error, log and continue rendering
                                        log_once::error_once!("Non-fatal error during frame prep: {}", e);

                                        None
                                    }
                                }
                            }
                        };

                        if let Some(frame_data) = frame_data {
                            let render_res = engine.renderer.render(&mut frame, frame_data);

                            match render_res {
                                Ok(_) => {
                                    // Rendered successfully
                                }
                                Err(e) => {
                                    match e {

                                        _ => {
                                            // Non-fatal error, log and continue rendering
                                            log_once::error_once!("Non-fatal error during rendering: {}", e);
                                        }
                                    }
                                }
                            }
                        }

                        Stats::global().set(
                            "CPU render time",
                            Stat::Duration(cpu_render_start.elapsed()),
                            false,
                        );

                        for (plugin, _, _) in plugins.get_plugins_in_order() {
                            let res = plugin.post_render(&mut engine, &mut frame, app);

                            if let Err(e) = res {
                                match e {
                                    PluginError::Fatal(e) => {
                                        log::error!(
                                            "Plugin '{}' failed in post_render: {:?}. Fatal error, stopping...",
                                            plugin.name(),
                                            e
                                        );
                                        engine.exit();
                                    }
                                    PluginError::NonFatal(e) => {
                                        log::error!(
                                            "Plugin '{}' failed in post_render: {:?}. Non-fatal error, continuing...",
                                            plugin.name(),
                                            e
                                        );
                                        continue;
                                    }
                                }
                            }
                        }

                        let gpu_render_start = std::time::Instant::now();

                        graphics
                            .end_frame(frame, Some(|| engine.window.winit().pre_present_notify()));

                        Stats::global().set(
                            "GPU render time",
                            Stat::Duration(gpu_render_start.elapsed()),
                            false,
                        );

                        Stats::global().set(
                            "Frametime",
                            Stat::Duration(cpu_render_start.elapsed()),
                            false,
                        );

                        Stats::global().set(
                            "1 / Frametime",
                            Stat::String(format!("{:.2}", 1.0 / cpu_render_start.elapsed().as_secs_f32())),
                            false,
                        );

                        last_second_frames += 1;

                        if last_avg_fps_update.elapsed() >= Duration::from_secs(1) {
                            Stats::global().set(
                                "Avg FPS", format!("{}", last_second_frames).into(), false);

                            last_avg_fps_update = std::time::Instant::now();
                            last_second_frames = 0;

                        }
                    }
                    WindowEvent::Resized(size) => {
                        let current_present_mode = Graphics::global_read().current_present_mode;

                        Graphics::global_write()
                            .configure_surface(size.into(), current_present_mode);

                        engine
                            .renderer
                            .resize_callback(size.into())
                            .unwrap_or_else(|e| {
                                log::error!("Failed to resize renderer: {:?}", e);
                            });

                        app.on_resize(&mut engine, &mut plugins, size.width, size.height);

                        for (plugin, _, _) in plugins.get_plugins_in_order() {
                            let res = plugin.on_resize(&mut engine, app);

                            if let Err(e) = res {
                                match e {
                                    PluginError::Fatal(e) => {
                                        log::error!(
                                            "Plugin '{}' failed in on_resize: {:?}. Fatal error, stopping...",
                                            plugin.name(),
                                            e
                                        );
                                        engine.exit();
                                    }
                                    PluginError::NonFatal(e) => {
                                        log::error!(
                                            "Plugin '{}' failed in on_resize: {:?}. Non-fatal error, continuing...",
                                            plugin.name(),
                                            e
                                        );
                                        continue;
                                    }
                                }
                            }
                        }

                        engine.window.winit().request_redraw();
                    }
                    _ => (),
                }
            }
            Event::AboutToWait => {
                engine.window.winit().request_redraw();
            }
            _ => (),
        };

        (*Stats::global()).frame_reset();
    })?;

    Ok(())
}
