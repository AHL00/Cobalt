mod unlit;
pub use unlit::*;

use crate::{graphics::{self, Graphics}, resource::ResourceTrait};


// NOTE: Every material should use the vertex type: UvNormalVertex

impl ResourceTrait for Material {}

pub enum Material {
    Unlit(Unlit),
}

impl MaterialTrait for Material {
    fn set_uniforms<'a>(&'a self, n: u32, render_pass: &mut wgpu::RenderPass<'a>, graphics: &Graphics) {
        match &self {
            Material::Unlit(unlit) => unlit.set_uniforms(n, render_pass, graphics),
        }
    }

    fn get_pipeline(&self) -> &'static wgpu::RenderPipeline {
        match &self {
            Material::Unlit(unlit) => unlit.get_pipeline(),
        }
    }
}

pub(crate) trait MaterialTrait {
    /// Set the uniforms for the material
    fn set_uniforms<'a>(&'a self, n: u32, render_pass: &mut wgpu::RenderPass<'a>, graphics: &Graphics);

    /// Get the render pipeline for the material
    fn get_pipeline(&self) -> &'static wgpu::RenderPipeline;
}