use std::error::Error;

use cobalt_core::graphics::{frame::Frame, winit::{self, event::WindowEvent}};

use crate::{app::App, engine::Engine};

use downcast::{downcast, Any};

/// Structs that allows more functionality to be implemented into the `Engine`.
/// Returning an error in any of the functions will stop the engine, unless the error is specified as non-fatal.
/// Warning: Do not call any of these functions directly, they are called by the engine.
pub trait Plugin: Any {
    /// Called once at the start of the engine.
    fn startup(&mut self, _engine: &mut Engine, _app: &mut dyn App) -> Result<(), PluginError> {
        Ok(())
    }

    /// Called at the start of a new frame.
    fn pre_render(&mut self, _engine: &mut Engine, _app: &mut dyn App) -> Result<(), PluginError> {
        Ok(())
    }

    /// Called after rendering is done but before frame submission.
    /// A `Frame` struct is passed to allow for more custom rendering.
    fn post_render(
        &mut self,
        _engine: &mut Engine,
        _frame: &mut Frame,
        _app: &mut dyn App,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    /// Called when an event is received. Plugins get access to events before the engine does.
    /// If the function returns `true`, the event will be consumed and not processed by the engine.
    fn window_event(
        &mut self,
        _engine: &mut Engine,
        _window_event: WindowEvent,
        _window_id: winit::window::WindowId,
        _app: &mut dyn App,
    ) -> Result<bool, PluginError> {
        Ok(false)
    }

    /// Called on window resize.
    fn on_resize(&mut self, _engine: &mut Engine, _app: &mut dyn App) -> Result<(), PluginError> {
        Ok(())
    }

    /// Called in the main event loop. Should be very fast, ideally this should not be
    /// used.
    fn update(&mut self, _engine: &mut Engine, _app: &mut dyn App) -> Result<(), PluginError> {
        Ok(())
    }

    /// Called once when the engine is shutting down.
    fn shutdown(&mut self, _engine: &mut Engine, _app: &mut dyn App) -> Result<(), PluginError> {
        Ok(())
    }

    fn name(&self) -> &'static str;
}

impl std::fmt::Debug for dyn Plugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Plugin")
            .field("name", &self.name())
            .finish()
    }
}

downcast!(dyn Plugin);

#[derive(Debug)]
pub enum PluginError {
    /// A fatal error will stop the engine.
    Fatal(Box<dyn Error>),
    /// A non-fatal error will not stop the engine, but will be logged at the error level.
    NonFatal(Box<dyn Error>),
}