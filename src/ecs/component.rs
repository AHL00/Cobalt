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

// TODO: Figure out blanket implementation for Component.
impl Component for String {}
impl Component for usize {}
impl Component for u8 {}
impl Component for u16 {}
impl Component for u32 {}
impl Component for u64 {}
impl Component for u128 {}
impl Component for isize {}
impl Component for i8 {}
impl Component for i16 {}
impl Component for i32 {}
impl Component for i64 {}
impl Component for i128 {}
impl Component for f32 {}
impl Component for f64 {}
impl Component for bool {}

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