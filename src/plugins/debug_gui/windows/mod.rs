use cobalt_core::stats::Stats;

mod assets;
mod heirarchy;
mod stats;
mod scene_serde;

/// Persistent data used to store debug menu state.
pub struct DebugMenu {
    show_stats: bool,
    stats: stats::StatsPanel,
    show_assets: bool,
    assets: assets::AssetsPanel,
    show_heirarchy: bool,
    heirarchy: heirarchy::HeirarchyPanel,
    show_scene_serde: bool,
    scene_serde: scene_serde::SerdePanel,
}

impl DebugMenu {
    pub fn new() -> Self {
        Self {
            show_stats: false,
            stats: stats::StatsPanel::new(),
            show_assets: false,
            assets: assets::AssetsPanel::new(),
            show_heirarchy: false,
            heirarchy: heirarchy::HeirarchyPanel::new(),
            show_scene_serde: false,
            scene_serde: scene_serde::SerdePanel::new(),
        }
    }

    pub fn show(
        &mut self,
        egui_ctx: &egui::Context,
        engine: &mut cobalt_runtime::engine::Engine,
        _app: &mut dyn cobalt_runtime::app::App,
    ) {
        egui::TopBottomPanel::top("debug_menu_bar").show(egui_ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Exit", |_| {
                    engine.request_exit();
                });

                ui.menu_button("Windows", |ui| {
                    ui.checkbox(&mut self.show_stats, "Stats");
                    ui.checkbox(&mut self.show_assets, "Assets");
                    ui.checkbox(&mut self.show_heirarchy, "Heirarchy");
                    ui.checkbox(&mut self.show_scene_serde, "Scene Serde");
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some((frame_time, _)) = Stats::global().get("Frametime") {
                        ui.label(format!("Frame time: {:.2}", frame_time));
                    }

                    if let Some((fps, _)) = Stats::global().get("Avg FPS") {
                        ui.label(format!("Avg FPS: {:.2}", fps));
                    }

                    if let Some((debug_mode, _)) = Stats::global().get("Geometry Pass Debug Mode") {
                        ui.label(format!("Debug mode: {}", debug_mode));
                    }
                });
            });
        });

        if self.show_stats {
            self.stats.show(egui_ctx);
        }

        if self.show_assets {
            self.assets.show(egui_ctx, engine);
        }

        if self.show_heirarchy {
            self.heirarchy.show(egui_ctx, engine);
        }

        if self.show_scene_serde {
            self.scene_serde.show(egui_ctx, engine);
        }
    }
}

pub mod exports {}
