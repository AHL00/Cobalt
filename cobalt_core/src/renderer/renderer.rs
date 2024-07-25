use downcast::{downcast, Any};

use crate::{exports::{ecs::{Entity, World}, renderer::Material}, graphics::frame::Frame};

use super::FrameData;

/// This is not implementable from outside core.
/// The exports will be messed up, see `renderer::exports`.
pub trait Renderer: Any {
    fn new(size: (u32, u32)) -> Result<Self, RendererError>
    where
        Self: Sized;

    /// Prepares necessary data for a frame. It should be called before rendering.
    /// All necessary world data will be copied and stored in the renderer.
    fn prep_frame<'a>(
        &mut self,
        frame: &mut Frame,
        world: &'a mut World,
        surface_dimensions: (u32, u32),
    ) -> Result<FrameData<'a, Material>, FramePrepError>;

    fn render(
        &mut self,
        frame: &mut Frame,
        frame_data: FrameData<Material>,
    ) -> Result<(), RendererError>;

    /// Should update current size, resize buffers, and send the callback along to all render passes.
    fn resize_callback(&mut self, size: (u32, u32)) -> Result<(), RendererError>;

    fn get_current_output_size(&self) -> (u32, u32);

    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

downcast!(dyn Renderer);

#[derive(thiserror::Error, Debug)]
pub enum FramePrepError {
    #[error("No camera entity found.")]
    NoCamera,
    #[error("Camera entity does not have a transform component.")]
    NoCamTransform,
    #[error("More than one enabled camera entity found.")]
    MultipleCameras,
    #[error("No renderables found.")]
    NoRenderables,
    #[error("Material not found on entity: {0}")]
    NoMaterial(Entity),
}

#[derive(thiserror::Error, Debug)]
pub enum RendererError {
    #[error("Render pass error: {0}")]
    RenderPassError(String),
    #[error("Resize error: {0}")]
    ResizeError(String),
    #[error("Buffer error: {0}")]
    BufferError(String),
}
