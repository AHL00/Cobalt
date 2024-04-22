use std::error::Error;

use cobalt_core::graphics::frame::Frame;

use crate::engine::Engine;

/// Structs that allows more functionality to be implemented into the `Engine`. 
/// Returning an error in any of the functions will stop the engine, unless the error is specified as non-fatal.
pub trait Plugin {
    /// Called once at the start of the engine.
    fn startup(&mut self, _engine: &mut Engine) -> Result<(), PluginError> {
        Ok(())
    }

    /// Called at the start of a new frame.
    fn pre_render(&mut self, _engine: &mut Engine) -> Result<(), PluginError> {
        Ok(())
    }

    /// Called after rendering is done but before frame submission.
    /// A `Frame` struct is passed to allow for more custom rendering.
    fn post_render(&mut self, _engine: &mut Engine, _frame: &mut Frame) -> Result<(), PluginError> {
        Ok(())
    }

    /// Called in the main event loop. Should be very fast, ideally this should not be
    /// used.
    fn update(&mut self, _engine: &mut Engine)  -> Result<(), PluginError> {
        Ok(())
    }

    /// Called once when the engine is shutting down.
    fn shutdown(&mut self, _engine: &mut Engine) -> Result<(), PluginError> {
        Ok(())
    }

    fn name(&self) -> &'static str;
}

pub enum PluginError {
    /// A fatal error will stop the engine.
    Fatal(Box<dyn Error>),
    /// A non-fatal error will not stop the engine, but will be logged at the error level.
    NonFatal(Box<dyn Error>),
}