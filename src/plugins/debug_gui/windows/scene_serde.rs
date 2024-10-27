use core::f32;

use cobalt_runtime::engine::Engine;

pub struct SerdePanel {
    last_ser_result: Option<String>,
}

impl SerdePanel {
    pub fn new() -> Self {
        Self {
            last_ser_result: None,
        }
    }

    pub fn show(&mut self, egui_ctx: &egui::Context, engine: &mut Engine) {
        egui::Window::new("Scene Serde").show(egui_ctx, |ui| {
            let scene = &engine.scene;

            ui.label(format!("Scene name: {:?}", scene.name));

            if ui.button("Serialize yaml").clicked() {
                let serialized = scene.serialize_yaml();
                if let Ok(serialized) = serialized {
                    self.last_ser_result = Some(serialized);
                } else {
                    self.last_ser_result = None;
                    log::error!("Failed to serialize scene: {:?}", serialized.unwrap_err());
                }
            }

            if let Some(ser_result) = &mut self.last_ser_result {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::TextEdit::multiline(ser_result)
                        .desired_rows(12)
                        .desired_width(f32::INFINITY)
                        .code_editor()
                        .show(ui);
                });
            }
        });
    }
}
