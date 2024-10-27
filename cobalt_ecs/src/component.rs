use std::any::Any;

/// This ID is used to identify a component type.
/// It is used for things like component masks.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ComponentId(pub u8);

/// A component is a piece of data that can be attached to an entity.
/// Any struct that implements this trait can be attached to an entity.
pub trait Component: Any + Sized {
    type SerContext<'a>;

    fn serialize<'se, S>(
        &self,
        context: Self::SerContext<'se>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer;

    type DeContext<'a>;

    fn deserialise<'de, D>(
        context: Self::DeContext<'de>,
        deserializer: D,
    ) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>;

    fn type_id(&self) -> std::any::TypeId
    where
        Self: 'static,
    {
        std::any::TypeId::of::<Self>()
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
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