use std::any::Any;

/// This ID is used to identify a component type.
/// It is used for things like component masks.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ComponentId(pub u8);

/// A component is a piece of data that can be attached to an entity.
/// Any struct that implements this trait can be attached to an entity.
pub trait Component: Any + sealed::SealedComponent {}

// Blanket implementation of ComponentInternal for all types that implement Component.
impl<T> sealed::SealedComponent for T where T: Component {}

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

pub(crate) mod sealed {
    use std::any::{Any, TypeId};

    pub trait SealedComponent {
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
