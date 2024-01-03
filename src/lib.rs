#![feature(generic_const_exprs)]
#![allow(dead_code)]

pub(crate) mod internal;
pub mod scene;
pub mod components;
pub mod engine;
pub mod ecs;

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