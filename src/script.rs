use crate::{
    ecs::{component::Component, Entity},
    engine::{DynApp, Engine},
};

pub trait Script {
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn update(&mut self, engine: &mut Engine, app: &mut DynApp, entity: Entity) {
        let _ = entity;
        let _ = app;
        let _ = engine;
    }

    fn on_load(&self, engine: &mut Engine, entity: Entity) {
        let _ = entity;
        let _ = engine;
    }
}

pub struct ScriptComponent {
    pub scripts: Vec<Box<dyn Script>>,
}

impl Component for ScriptComponent {}

impl ScriptComponent {
    pub fn empty() -> Self {
        Self {
            scripts: Vec::new(),
        }
    }

    pub fn with_scripts(scripts: Vec<Box<dyn Script>>) -> Self {
        Self { scripts }
    }

    pub fn add_script<S: Script + 'static>(&mut self, script: S) {
        self.scripts.push(Box::new(script));
    }
}
