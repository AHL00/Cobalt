use std::fmt::{self, Display, Formatter};

use crate::{exports::Component, utils::bit_array::SimdBitArray};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Entity {
    pub(crate) id: u32,
    /// The version is used to check if an entity is still valid.
    /// When a entity is deleted, the id stays the same but the version is incremented.
    /// This id is then reused for new entities.
    /// This allows for easy recycling of entities without having to worry about dangling references.
    /// This can become a u16 if we need more data in this struct
    pub(crate) version: u32,
}

unsafe impl Send for Entity {}

impl Display for Entity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Entity({})", self.id())
    }
}

impl Entity {
    // The ID given out to users is actually a combination of the id and version.
    pub fn id(&self) -> u64 {
        (self.id as u64) << 32 | self.version as u64
    }

    pub fn get_component<T: 'static + Component>(self, world: &crate::world::World) -> Option<&T> {
        world.get_component::<T>(self)
    }

    pub fn add_component<T: 'static + Component>(self, world: &mut crate::world::World, component: T) {
        world.add_component(self, component);
    }

    pub fn remove_component<T: 'static + Component>(self, world: &mut crate::world::World) {
        world.remove_component::<T>(self);
    }

    pub fn get_component_mut<T: 'static + Component>(self, world: &mut crate::world::World) -> Option<&mut T> {
        world.get_component_mut::<T>(self)
    }

    pub fn has_component<T: 'static + Component>(self, world: &crate::world::World) -> Option<bool> {
        world.has_component::<T>(self)
    }
}

/// This struct holds data pertaining to a single entity. Basically an internal representation of an entity.
#[derive(Clone, Debug)]
pub(crate) struct EntityData {
    pub components: SimdBitArray<256>,
    pub version: u32,
    pub id: u32,
}
