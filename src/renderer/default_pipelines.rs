use crate::renderer::{mesh::MeshPipeline, sprite::SpritePipeline};

use super::Renderer;

impl Renderer {
    pub(crate) fn add_default_pipelines(&mut self) {
        let graphics = crate::engine::graphics();

        self.add_pipeline(SpritePipeline::new(&graphics));
        self.add_pipeline(MeshPipeline::new(&graphics));
    }
}
