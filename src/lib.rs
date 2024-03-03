#![feature(generic_const_exprs)]
#![feature(portable_simd)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(lazy_cell)]
#![allow(dead_code)]

pub(crate) mod internal;
pub mod scene;
pub mod engine;
pub mod ecs;
pub mod assets;
pub mod graphics;
pub mod renderer;
pub mod script;
pub mod input;
pub mod stats;
pub mod transform;

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