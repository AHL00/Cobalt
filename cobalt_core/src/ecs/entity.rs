use crate::utils::bit_array::SimdBitArray;


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

impl Entity {
    // The ID given out to users is actually a combination of the id and version.
    pub fn id(&self) -> u64 {
        (self.id as u64) << 32 | self.version as u64
    }
}

/// This struct holds data pertaining to a single entity. Basically an internal representation of an entity.
#[derive(Clone, Debug)]
pub(crate) struct EntityData {
    pub components: SimdBitArray<256>,
    pub version: u32,
    pub id: u32,
}