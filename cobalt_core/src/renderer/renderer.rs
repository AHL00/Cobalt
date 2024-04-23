use std::error::Error;

use crate::{exports::ecs::World, graphics::frame::Frame};

// TODO: Maybe split into internal and external, selectively exposing some methods.
pub trait Renderer {
    fn render(&mut self, frame: &mut Frame, world: &mut World) -> Result<(), Box<dyn std::error::Error>>;

    /// Should update current size, resize buffers, and send the callback along to all render passes.
    fn resize_callback(&mut self, size: (u32, u32)) -> Result<(), Box<dyn Error>>;

    fn get_current_output_size(&self) -> (u32, u32);

    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}