#![feature(generic_const_exprs)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![allow(dead_code)]

pub(crate) mod internal;
pub mod scene;
pub mod engine;
pub mod ecs;
pub mod assets;

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