use std::sync::OnceLock;

use super::texture::{Texture, TextureType};

/// Storage for various items that are initialized once, static, and reused.
pub struct GraphicsCache {
    pub bind_group_layout_cache: BindGroupLayoutCache,
    pub texture_cache: TextureCache,
    pub buffer_cache: BufferCache,
}

impl GraphicsCache {
    pub fn new() -> Self {
        Self {
            bind_group_layout_cache: BindGroupLayoutCache::new(),
            texture_cache: TextureCache::new(),
            buffer_cache: BufferCache::new(),
        }
    }
}

pub struct BindGroupLayoutCache {
    pub transform: OnceLock<wgpu::BindGroupLayout>,
    pub mat4: OnceLock<wgpu::BindGroupLayout>,
    pub u32: OnceLock<wgpu::BindGroupLayout>,
    pub vec3: OnceLock<wgpu::BindGroupLayout>,
    pub proj_view: OnceLock<wgpu::BindGroupLayout>,
    pub material: OnceLock<wgpu::BindGroupLayout>,
    pub depth_buffer: OnceLock<wgpu::BindGroupLayout>,
    pub g_buffer: OnceLock<wgpu::BindGroupLayout>,
    pub textures: TexturesBindGroupLayoutCache,
}

impl BindGroupLayoutCache {
    pub fn new() -> Self {
        Self {
            transform: OnceLock::new(),
            mat4: OnceLock::new(),
            u32: OnceLock::new(),
            vec3: OnceLock::new(),
            proj_view: OnceLock::new(),
            material: OnceLock::new(),
            depth_buffer: OnceLock::new(),
            g_buffer: OnceLock::new(),
            textures: TexturesBindGroupLayoutCache::new(),
        }
    }
}

pub struct TexturesBindGroupLayoutCache {
    pub rgba32_float: OnceLock<wgpu::BindGroupLayout>,
    pub rgba16_float: OnceLock<wgpu::BindGroupLayout>,
    pub rgba8_unorm: OnceLock<wgpu::BindGroupLayout>,
    pub rgba8_unorm_srgb: OnceLock<wgpu::BindGroupLayout>,
    pub r32_float: OnceLock<wgpu::BindGroupLayout>,
    pub r16_float: OnceLock<wgpu::BindGroupLayout>,
    pub r8_unorm: OnceLock<wgpu::BindGroupLayout>,
    pub r8_uint: OnceLock<wgpu::BindGroupLayout>,
    pub r8_snorm: OnceLock<wgpu::BindGroupLayout>,
}

impl TexturesBindGroupLayoutCache {
    pub fn new() -> Self {
        Self {
            rgba32_float: OnceLock::new(),
            rgba16_float: OnceLock::new(),
            rgba8_unorm: OnceLock::new(),
            rgba8_unorm_srgb: OnceLock::new(),
            r32_float: OnceLock::new(),
            r16_float: OnceLock::new(),
            r8_unorm: OnceLock::new(),
            r8_uint: OnceLock::new(),
            r8_snorm: OnceLock::new(),
        }
    }
}

pub struct TextureCache {
    pub(super) empty_rgba32_float: OnceLock<Texture::<{TextureType::RGBA32Float}>>,
    pub(super) empty_rgba16_float: OnceLock<Texture<{TextureType::RGBA16Float}>>,
    pub(super) empty_rgba8_unorm: OnceLock<Texture<{TextureType::RGBA8Unorm}>>,
    pub(super) empty_rgba8_unorm_srgb: OnceLock<Texture<{TextureType::RGBA8UnormSrgb}>>,
    pub(super) empty_r32_float: OnceLock<Texture<{TextureType::R32Float}>>,
    pub(super) empty_r16_float: OnceLock<Texture<{TextureType::R16Float}>>,
    pub(super) empty_r8_unorm: OnceLock<Texture<{TextureType::R8Unorm}>>,
    pub(super) empty_r8_uint: OnceLock<Texture<{TextureType::R8Uint}>>,
    pub(super) empty_r8_snorm: OnceLock<Texture<{TextureType::R8Snorm}>>,
}

pub struct BufferCache {
    pub plane_vertex_buffer: OnceLock<wgpu::Buffer>,
    pub plane_index_buffer: OnceLock<wgpu::Buffer>,

    pub screen_quad_vertex_buffer: OnceLock<wgpu::Buffer>,
    pub screen_quad_index_buffer: OnceLock<wgpu::Buffer>,
}

impl BufferCache {
    pub fn new() -> Self {
        Self {
            plane_vertex_buffer: OnceLock::new(),
            plane_index_buffer: OnceLock::new(),
            screen_quad_index_buffer: OnceLock::new(),
            screen_quad_vertex_buffer: OnceLock::new(),
        }
    }
}
