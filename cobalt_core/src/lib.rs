#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(adt_const_params)]
#![feature(portable_simd)]
// #![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(lazy_cell)]


pub mod utils;
/// A global asset system.
pub mod assets;
pub mod components;
pub mod ecs;
/// Graphics abstractions such as windows, textures, etc.
pub mod graphics;
pub mod input;
pub mod renderer;
pub mod scenes;
pub mod stats;
pub mod types;
/// Maths re-exported from ultraviolet.
pub mod maths {
    pub use ultraviolet::{rotor, transform, vec, Rotor2, Rotor3, Vec2, Vec3, Vec4};
}

pub mod exports {
    use super::*;

    pub use super::maths;
    pub use assets::exports as assets;
    pub use components::exports as components;
    pub use ecs::exports as ecs;
    pub use graphics::exports as graphics;
    pub use input::exports as input;
    pub use renderer::exports as renderer;
    pub use stats::exports as stats;
    pub use types::exports as types;
    pub use scenes::exports as scenes;
}
