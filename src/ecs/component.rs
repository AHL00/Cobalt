use std::any::Any;

use serde::{Serialize, Deserialize};

/// A component is a piece of data that can be attached to an entity.
/// Any struct that implements this trait can be attached to an entity.
pub trait Component: Serialize + Deserialize<'static> + Any + internal::ComponentInternal {

}

// Blanket implementation of ComponentInternal for all types that implement Component.
impl<T> internal::ComponentInternal for T
where T: Component
{}

pub(crate) mod internal {
    use std::any::{Any, TypeId};

    pub trait ComponentInternal {
        fn type_id(&self) -> TypeId
        where
            Self: 'static,
        {
            TypeId::of::<Self>()
        }
    
        fn as_any(&self) -> &dyn Any
        where
            Self: Sized + 'static,
        {
            self
        }
    
        fn as_any_mut(&mut self) -> &mut dyn Any
        where
            Self: Sized + 'static,
        {
            self
        }
    }
}