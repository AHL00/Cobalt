use std::{
    any::TypeId,
    sync::{Arc, OnceLock},
};

use hashbrown::HashMap;
use parking_lot::RwLock;

use super::texture::{Texture, TextureType};

/// Storage for various items that are initialized once, static, and reused.
pub(crate) struct GraphicsCache {
    pub bind_group_layout_cache: Arc<RwLock<HashMap<TypeId, wgpu::BindGroupLayout>>>,
    pub texture_cache: TextureCache,
    pub buffer_cache: BufferCache,
}

impl GraphicsCache {
    pub fn new() -> Self {
        Self {
            bind_group_layout_cache: Arc::new(RwLock::new(HashMap::new())),
            texture_cache: TextureCache::new(),
            buffer_cache: BufferCache::new(),
        }
    }
}

pub(crate) struct TextureCache {
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

pub(crate) struct BufferCache {
    pub(crate) plane_vertex_buffer: OnceLock<wgpu::Buffer>,
    pub(crate) plane_index_buffer: OnceLock<wgpu::Buffer>,

    pub(crate) screen_quad_vertex_buffer: OnceLock<wgpu::Buffer>,
    pub(crate) screen_quad_index_buffer: OnceLock<wgpu::Buffer>,
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
