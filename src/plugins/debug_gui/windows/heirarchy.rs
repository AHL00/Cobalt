use cobalt_core::exports::components::EntityName;
use cobalt_runtime::engine::Engine;

pub struct HeirarchyPanel {
    selected_entity: Option<crate::ecs::Entity>,
    scroll: egui::ScrollArea,
}

impl HeirarchyPanel {
    pub fn new() -> Self {
        Self {
            selected_entity: None,
            scroll: egui::ScrollArea::vertical(),
        }
    }

    pub fn show(&mut self, egui_ctx: &egui::Context, engine: &mut Engine) {
        egui::Window::new(format!("Heirarchy [{}]", engine.scene.name)).show(egui_ctx, |ui| {
            let world = &mut engine.scene.world;

            // let mut entity_tree = EntityTree::new(world);

            // entity_tree.show(ui, &mut self.selected_entity);

            // if let Some(selected_entity) = self.selected_entity {
            //     ui.separator();

            //     if ui.button("Select").clicked() {
            //         engine.select_entity(selected_entity);
            //     }

            //     ui.separator();

            //     if ui.button("Delete").clicked() {
            //         engine.delete_entity(selected_entity);
            //     }
            // }

            // Heirarchy not implemented yet
            // So, we just list all entities in the scene with their respective components
            let mut remove_list = vec![];

            for entity in world.entities() {
                let entity_name: String = entity
                    .get_component(world)
                    .map(|name: &EntityName| name.into())
                    .unwrap_or("Unnamed Entity".to_string());

                let mut components = vec![];

                for component in world.list_components(entity) {
                    components.push(component);
                }

                ui.push_id(entity.id(), |ui| {
                    ui.collapsing(format!("{}", entity_name), |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Delete").clicked() {
                                remove_list.push(entity);
                            }
                        });
                        for component in components {
                            ui.label(format!("{:?}", component));
                        }
                    });
                });
            }

            for entity in remove_list {
                engine.scene.world.remove_entity(entity);
            }
        });
    }
}
