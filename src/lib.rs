#![feature(generic_const_exprs)]
#![feature(portable_simd)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(lazy_cell)]
#![allow(dead_code)]

pub mod assets;
pub mod ecs;
pub mod engine;
pub mod graphics;
pub mod input;
pub(crate) mod internal;
pub mod renderer;
pub mod scene;
pub mod script;
pub mod stats;
pub mod transform;
#[cfg(feature = "dev_gui")]
pub mod dev_gui;

// Tell NVIDIA Optimus to use high performance GPU.
#[cfg(target_os = "windows")]
#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn NvOptimusEnablement() -> i32 {
    1
}

// Tell AMD PowerXpress to use high performance GPU.
#[cfg(target_os = "windows")]
#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn AmdPowerXpressRequestHighPerformance() -> i32 {
    1
}
