use cobalt_core::stats::Stats;

pub struct StatsPanel {
    pub displayed_stats: std::collections::HashMap<String, bool>,
}

impl StatsPanel {
    pub fn new() -> Self {
        Self {
            displayed_stats: std::collections::HashMap::new(),
        }
    }


pub fn show(&mut self, egui_ctx: &egui::Context) {
    egui::Window::new("Stats").show(egui_ctx, |ui| {
        let s = Stats::global();
        let stats = s.sorted_by_label();

        // If there is a new stat, replace the hashmap with the vec
        let mut stats_dirty = false;
        for (name, _stat) in &stats {
            if !self.displayed_stats.contains_key(name.as_str()) {
                stats_dirty = true;
            }
        }

        if stats_dirty {
            let old_displayed_stats = self.displayed_stats.clone();

            self.displayed_stats.clear();

            for (name, _stat) in &stats {
                self.displayed_stats.insert(
                    (*name).clone(),
                    if old_displayed_stats.contains_key(*name) {
                        *old_displayed_stats.get(*name).unwrap()
                    } else {
                        false
                    },
                );
            }
        }

        egui::CollapsingHeader::new("Enabled stats").show(ui, |ui| {

            ui.horizontal(|ui| {
                if ui.button("Enable all").clicked() {
                    for (name, _) in &stats {
                        self.displayed_stats.insert((*name).clone(), true);
                    }
                }
                if ui.button("Disable all").clicked() {
                    for (name, _) in &stats {
                        self.displayed_stats.insert((*name).clone(), false);
                    }
                }
            });

            ui.separator();

            for (name, _stat) in &stats {
                ui.checkbox(
                    &mut self.displayed_stats.get_mut(*name).unwrap(),
                    *name,
                );
            }
        });

        ui.separator();

        egui::Grid::new("stats_grid")
            .striped(true)
            .num_columns(2)
            .spacing([10.0, 10.0])
            .show(ui, |ui| {
                for (name, stat) in &stats {
                    if *self.displayed_stats.get(*name).unwrap() {
                        ui.label(format!("{}: ", *name)).highlight();
                        ui.label(stat.to_string());
                        ui.end_row();
                    }
                }
            });
    });
}}