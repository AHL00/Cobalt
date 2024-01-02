#![feature(generic_const_exprs)]
#![allow(dead_code)]

use component::{Component, internal::ComponentInternal};
use serde::{Serialize, Deserialize};

pub(crate) mod internal;
pub mod scene;
pub mod components;
pub mod component;
pub mod engine;
pub mod ecs;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct TestComponent {
    pub name: String,
}

#[typetag::serde]
impl Component for TestComponent {
    fn name(&self) -> &str {
        "TestComponent"
    }
}

impl ComponentInternal for TestComponent {
    fn on_load(&mut self) {
        println!("TestComponent loaded.");
    }

    fn on_unload(&mut self) {
        println!("TestComponent unloaded.");
    }

    fn on_update(&mut self, _delta_time: f32) {
        println!("TestComponent updated.");
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct TestComponent2 {
    pub name: String,
}

#[typetag::serde]
impl Component for TestComponent2 {
    fn name(&self) -> &str {
        "TestComponent2"
    }
}

impl ComponentInternal for TestComponent2 {
    fn on_load(&mut self) {
        println!("TestComponent2 loaded.");
    }

    fn on_unload(&mut self) {
        println!("TestComponent2 unloaded.");
    }

    fn on_update(&mut self, _delta_time: f32) {
        println!("TestComponent2 updated.");
    }
}

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