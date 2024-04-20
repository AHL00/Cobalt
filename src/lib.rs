#![feature(generic_const_exprs)]
#![feature(portable_simd)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(lazy_cell)]
#![allow(dead_code)]

pub mod assets;
pub mod resource;
pub mod ecs;
pub mod engine;
pub mod graphics;
pub mod input;
/// This is made public for flexibility, but it is not recommended to use it.
pub mod internal;
pub mod renderer;
pub mod scene;
pub mod script;
pub mod stats;
pub mod transform;

#[cfg(feature = "dev_gui")]
pub mod dev_gui;

// Maths
pub mod maths {
    pub use ultraviolet::{Vec2, Vec3, Vec4, Rotor3, Rotor2, rotor, vec, transform};
}