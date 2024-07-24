use cobalt_core::assets::server::AssetServer;
use egui_extras::Column;

pub struct AssetsPanel {}

impl AssetsPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, egui_ctx: &egui::Context) {
        egui::Window::new("Assets").show(egui_ctx, |ui| {
            let asset_server = AssetServer::global_read();

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
                            ui.label("Packed");
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
                                    ui.label(if asset.packed.is_some() { "Yes" } else { "No" });
                                });

                                row.col(|ui| {
                                    ui.label(if asset.packed.is_some() {
                                        if let Some(level) =
                                            asset.packed.as_ref().unwrap().compression
                                        {
                                            format!("{:?}", level)
                                        } else {
                                            "No".to_string()
                                        }
                                    } else {
                                        "N/A".to_owned()
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
