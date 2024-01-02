use std::any::Any;

/// A component is a piece of data that can be attached to an entity.
/// Any struct that implements this trait can be attached to an entity.
#[typetag::serde(tag = "type")]
pub trait Component: Any + Send + Sync + internal::ComponentInternal {
    /// Returns the name of the component.
    fn name(&self) -> &str;
}

pub(crate) mod internal {
    use crate::internal::as_any::AsAny;

    pub trait ComponentInternal: AsAny {
        /// Called when the component is loaded.
        fn on_load(&mut self) {}
    
        /// Called when the component is unloaded.
        fn on_unload(&mut self) {}
    
        /// Called when the component is updated.
        /// The delta time is passed in as a parameter.
        /// This is called once per frame.
        fn on_update(&mut self, _delta_time: f32) {}
    }
}