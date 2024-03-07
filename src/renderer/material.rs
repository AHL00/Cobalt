use ultraviolet::Vec4;

use crate::{assets::AssetHandle, graphics::{texture::TextureAsset, CreateBindGroup, HasBindGroup, HasBindGroupLayout}};

// TODO: Figure out how this works
// https://github.com/takahirox/wgpu-rust-renderer/blob/main/src/material/material.rs
// Maybe like this code?
pub(crate) trait MaterialTrait: HasBindGroupLayout + HasBindGroup {

}

pub enum Material {
    PBR(PBR),
    Unlit(Unlit),
}

pub struct PBR {
    pub albedo: Vec4,
    pub albedo_texture: Option<AssetHandle<TextureAsset>>,
    pub specular: f32,
    pub roughness: f32,
    pub metallic: f32,
    pub normal_map: Option<AssetHandle<TextureAsset>>,
}

impl Default for PBR {
    fn default() -> Self {
        Self {
            albedo: Vec4::one(),
            albedo_texture: None,
            specular: 0.0,
            roughness: 0.0,
            metallic: 0.0,
            normal_map: None,
        }
    }
}

pub struct Unlit {
    pub color: Vec4,
    pub texture: Option<AssetHandle<TextureAsset>>,

    bind_group: Option<wgpu::BindGroup>,
    bind_group_layout: Option<wgpu::BindGroupLayout>,
}

impl Unlit {
    pub fn new(color: Vec4, texture: Option<AssetHandle<TextureAsset>>) -> Self {
        let mut res = Self {
            color,
            texture,
            bind_group: None,
            bind_group_layout: None,
        };

        // This will create the bind group layout and bind group
        let _ = res.bind_group();

        res
    }
}

impl MaterialTrait for Unlit {}

impl HasBindGroupLayout for Unlit {
    fn bind_group_layout() -> &'static wgpu::BindGroupLayout {
        todo!()
    }
}

impl HasBindGroup for Unlit {
    fn bind_group(&mut self) -> &wgpu::BindGroup {
        todo!()
    }
}


impl Default for Unlit {
    fn default() -> Self {
        Self::new(Vec4::one(), None)
    }
}

