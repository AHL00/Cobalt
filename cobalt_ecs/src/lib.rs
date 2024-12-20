#![feature(portable_simd)]
#![feature(generic_const_exprs)]
#![feature(fn_traits)]

pub mod exports {
    pub use super::{component::Component, entity::Entity, world::World};

    pub mod query {
        pub use super::super::query::exports::*;
    }
}

pub mod component;
pub mod entity;
pub mod query;
pub mod storage;
pub mod typeid_map;
pub mod world;
pub mod utils;

mod tests;

// ## Problems
// - Performance issues due to use of hash maps

// ## Plan
// - Rewrite with more experience
// - This time, use the following instead of HashMaps
//     - Two arrays:
//       - Dense array of X
//       - Sparse array of dense array indices where the index is the entity id
//   - This will allow for cache friendly iteration
//   - This data structure is called a "sparse set"
//   - Introduce versioning to entities

// ## Notes
// - Entity IDs will be 32 bit unsigned integers
// - Entities will now be versioned to allow for easy recycling of IDs
// - Component IDs will be 8 bit unsigned integers.
//   This means that there can only be 256 component types.
//   This should be more than enough for most use cases.
