use cobalt_core::stats::Stats;

use self::stats::StatsWindow;

mod entities;
mod menu_bar;
mod stats;

/// Persistent data used to store debug menu state.
pub struct DebugMenu {
    show_stats: bool,
    stats: stats::StatsWindow,
}

impl DebugMenu {
    pub fn new() -> Self {
        Self {
            show_stats: false,
            stats: stats::StatsWindow::new(),
        }
    }

    pub fn show(
        &mut self,
        egui_ctx: &egui::Context,
        engine: &mut cobalt_runtime::engine::Engine,
        app: &mut dyn cobalt_runtime::app::App,
    ) {
        egui::TopBottomPanel::top("debug_menu_bar").show(egui_ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Exit", |ui| {
                        engine.exit();
                });

                ui.menu_button("Windows", |ui| {
                    ui.checkbox(&mut self.show_stats, "Stats");
                });

                if let Some((frame_time, _)) = Stats::global().get("Frametime") {
                    ui.label(format!("Frame time: {:.2}", frame_time));
                }

                if let Some((debug_mode, _)) = Stats::global().get("Geometry Pass Debug Mode") {
                    ui.label(format!("Debug mode: {}", debug_mode));
                }
            });
        });

        if self.show_stats {
            self.stats.show(egui_ctx);
        }
    }
}

pub mod exports {}
