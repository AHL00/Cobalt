#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(adt_const_params)]
#![feature(fn_traits)]
#![feature(lazy_cell)]


pub mod utils;
pub mod components;
pub mod input;
pub mod renderer;
pub mod scenes;
pub mod stats;
pub mod types;
pub mod assets_types;
pub use cobalt_graphics as graphics;
pub use cobalt_ecs as ecs;
pub use cobalt_assets as assets;

/// Maths re-exported from ultraviolet.
pub mod maths {
    pub use ultraviolet::{rotor, transform, vec, Rotor2, Rotor3, Vec2, Vec3, Vec4};
}
pub mod reexports {
    pub use ultraviolet;
    pub use wgpu;
    pub use winit;
}

pub use gltf;

pub mod exports {
    use super::*;

    pub use super::maths;
    pub use cobalt_assets::exports as assets;
    pub use components::exports as components;
    pub use cobalt_ecs::exports as ecs;
    pub use cobalt_graphics::exports as graphics;
    pub use input::exports as input;
    pub use renderer::exports as renderer;
    pub use stats::exports as stats;
    pub use types::exports as types;
    pub use scenes::exports as scenes;
    pub use assets_types::exports as asset_types;
}
