/// The RenderPass should not alter FrameData.
/// They are only mutable to allow for bind groups to be updated.
///
/// RenderPass's are responsible for drawing a single pass of the frame.
/// They can contain things such as geometry buffers, shadow maps, etc.
pub trait RenderPass {
    /// Creates render pass, and draws to the swap texture if the pass requires it.
    /// NOTE: `FrameData` is only mutable to allow for bind groups to be updated, e.g. `Transform::bind_group()`.
    fn draw(
        &mut self,
        frame: &mut crate::graphics::frame::Frame,
        graphics: &crate::graphics::context::Graphics,
        proj_view: crate::renderer::proj_view::ProjView,
        frame_data: &mut crate::renderer::FrameData,
    ) -> Result<(), Box<dyn std::error::Error>>;

    fn resize_callback(&mut self, size: (u32, u32)) -> Result<(), Box<dyn std::error::Error>>;

    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}
