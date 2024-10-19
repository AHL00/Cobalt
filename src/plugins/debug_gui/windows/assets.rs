use cobalt_runtime::engine::Engine;
use egui_extras::Column;

pub struct AssetsPanel {}

impl AssetsPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, egui_ctx: &egui::Context, engine: &mut Engine) {
        egui::Window::new("Assets").show(egui_ctx, |ui| {
            let asset_server = engine.assets();

            ui.label(format!("Assets directory: {:?}", asset_server.assets_dir()));

            egui::CollapsingHeader::new("Asset Manifest").show(ui, |ui| {
                egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .vscroll(true)
                    .resizable(true)
                    .columns(Column::auto(), 7)
                    .header(12.0, |mut row| {
                        row.col(|ui| {
                            ui.label("Asset ID");
                        });

                        row.col(|ui| {
                            ui.label("Name");
                        });

                        row.col(|ui| {
                            ui.label("Type");
                        });

                        row.col(|ui| {
                            ui.label("Path");
                        });

                        row.col(|ui| {
                            ui.label("Compression");
                        });

                        row.col(|ui| {
                            ui.label("Timestamp");
                        });
                    })
                    .body(|mut body| {
                        let manifest = asset_server.get_manifest().unwrap();

                        for asset in manifest.assets.iter() {
                            body.row(1.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(format!("{:?}", asset.asset_id.uuid()));
                                });

                                row.col(|ui| {
                                    ui.label(asset.name.as_str());
                                });

                                row.col(|ui| {
                                    ui.label(asset.type_name.as_str());
                                });

                                row.col(|ui| {
                                    ui.label(asset.relative_path.to_str().unwrap());
                                });

                                row.col(|ui| {
                                    ui.label(if let Some(level) = asset.pack.compression {
                                        format!("{:?}", level)
                                    } else {
                                        "No".to_string()
                                    });
                                });

                                row.col(|ui| {
                                    let timestamp = humantime::Timestamp::from(asset.timestamp);
                                    ui.label(format!("{}", timestamp));
                                });
                            });
                        }
                    });
            });
        });
    }
}
