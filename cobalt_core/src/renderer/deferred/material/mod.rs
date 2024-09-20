use std::sync::{atomic::AtomicUsize, Arc, Weak};

use bytes::Bytes;
use parking_lot::RwLock;
use wgpu::util::DeviceExt;

use crate::{assets_types::texture::TextureAsset, exports::types::{either::Either, resource::ResourceTrait}};
use cobalt_assets::{
    asset::AssetFileSystemType,
    exports::{Asset, AssetTrait},
};
use cobalt_graphics::{
    context::Graphics,
    texture::{TextureInternal, TextureType},
    HasBindGroupLayout,
};

static MATERIAL_ID: AtomicUsize = AtomicUsize::new(0);

/// Deferred renderer material.
pub struct Material {
    id: usize,

    /// If unlit is true, the material will not be affected by lighting.
    unlit: bool,

    /// If a color is set, the material will be rendered as a wireframe with that color.
    wireframe: Option<[f32; 4]>,

    /// The base color of th`e material.
    /// If only one of either the texture or color are set, it will be used.
    /// If both are set, the texture's color will be multiplied by the color.
    albedo: (
        Option<[f32; 4]>,
        Option<Asset<TextureAsset<{ Material::ALBEDO_TEXTURE_TYPE }>>>,
    ),

    /// Normal map, adds bumps and details to the surface.
    normal: Option<Asset<TextureAsset<{ Material::NORMAL_TEXTURE_TYPE }>>>,

    /// Metallic map or value.
    metallic: Either<f32, Asset<TextureAsset<{ Material::METALLIC_TEXTURE_TYPE }>>>,

    /// Roughness map or value.
    roughness: Either<f32, Asset<TextureAsset<{ Material::ROUGHNESS_TEXTURE_TYPE }>>>,

    bind_group: Option<wgpu::BindGroup>,

    graphics_weak_ref: Weak<RwLock<Graphics>>,
}

impl std::fmt::Debug for Material {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(format!("Material [id: {}]", self.id).as_str())
            .field("unlit", &self.unlit)
            .field("wireframe", &self.wireframe)
            .field("albedo", &self.albedo)
            .field("normal", &self.normal)
            .field("metallic", &self.metallic)
            .field("roughness", &self.roughness)
            .finish()
    }
}

impl Material {
    // NOTE: Make sure this type matches the EMPTY_? in the generate_bind_group function when changing.
    const ALBEDO_TEXTURE_TYPE: TextureType = TextureType::RGBA8UnormSrgb;
    const NORMAL_TEXTURE_TYPE: TextureType = TextureType::RGBA16Float;
    const METALLIC_TEXTURE_TYPE: TextureType = TextureType::R8Unorm;
    const ROUGHNESS_TEXTURE_TYPE: TextureType = TextureType::R8Unorm;

    fn generate_bind_group(&mut self) {
        let graphics_arc = self
            .graphics_weak_ref
            .upgrade()
            .expect("Graphics context dropped.");
        let graphics = graphics_arc.read();

        let unlit_buffer = graphics
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[self.unlit as u32]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let wireframe_buffer =
            graphics
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&[self.wireframe.is_some() as u32]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let wireframe_color_buffer =
            graphics
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&self.wireframe.unwrap_or([0.0, 0.0, 0.0, 0.0])),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let albedo_supplied = if self.albedo.0.is_some() && self.albedo.1.is_some() {
            BindingSuppliedType::Both
        } else if self.albedo.0.is_some() {
            BindingSuppliedType::Texture
        } else if self.albedo.1.is_some() {
            BindingSuppliedType::Value
        } else {
            panic!("Both albedo color and texture are None.");
        };

        let albedo_supplied_buffer =
            graphics
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&[albedo_supplied as u32]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let albedo_color_buffer =
            graphics
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&self.albedo.0.unwrap_or([1.0, 1.0, 1.0, 1.0])),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let albedo_texture = self.albedo.1.as_ref();

        let normal_supplied_buffer =
            graphics
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&[self.normal.is_some() as u32]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let normal_texture = self.normal.as_ref();

        let metallic_type = if self.metallic.is_left() {
            BindingSuppliedType::Texture
        } else {
            BindingSuppliedType::Value
        };

        let metallic_type_buffer =
            graphics
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&[metallic_type as u32]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let metallic_value_buffer =
            graphics
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&[self.metallic.left().map_or(1f32, |x| *x)]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let metallic_texture = self.metallic.right();

        let roughness_type = if self.roughness.is_left() {
            BindingSuppliedType::Texture
        } else {
            BindingSuppliedType::Value
        };

        let roughness_type_buffer =
            graphics
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&[roughness_type as u32]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let roughness_value_buffer =
            graphics
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&[self.roughness.left().map_or(1f32, |x| *x)]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let roughness_texture = self.roughness.right();

        let layout = graphics
            .cache
            .bind_group_layout_cache
            .material
            .get_or_init(|| create_material_bind_group_layout(&graphics.device));

        self.bind_group = Some(
            graphics
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Material Bind Group"),
                    layout: &layout,
                    entries: &[
                        // Unlit
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(
                                unlit_buffer.as_entire_buffer_binding(),
                            ),
                        },
                        // Wireframe
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Buffer(
                                wireframe_buffer.as_entire_buffer_binding(),
                            ),
                        },
                        // Wireframe color
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Buffer(
                                wireframe_color_buffer.as_entire_buffer_binding(),
                            ),
                        },
                        // Albedo supplied
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Buffer(
                                albedo_supplied_buffer.as_entire_buffer_binding(),
                            ),
                        },
                        // Albedo color
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: wgpu::BindingResource::Buffer(
                                albedo_color_buffer.as_entire_buffer_binding(),
                            ),
                        },
                        // Albedo texture
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: wgpu::BindingResource::TextureView(
                                albedo_texture.map_or(
                                    graphics
                                        .cache
                                        .texture_cache
                                        .empty_rgba8_unorm_srgb(&graphics)
                                        .wgpu_texture_view(),
                                    |x| unsafe { x.borrow_unsafe().0.wgpu_texture_view() },
                                ),
                            ),
                        },
                        // Albedo sampler
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: wgpu::BindingResource::Sampler(
                                albedo_texture.map_or(
                                    graphics
                                        .cache
                                        .texture_cache
                                        .empty_rgba8_unorm(&graphics)
                                        .wgpu_sampler(),
                                    |x| unsafe { x.borrow_unsafe().0.wgpu_sampler() },
                                ),
                            ),
                        },
                        // Normal supplied
                        wgpu::BindGroupEntry {
                            binding: 7,
                            resource: wgpu::BindingResource::Buffer(
                                normal_supplied_buffer.as_entire_buffer_binding(),
                            ),
                        },
                        // Normal texture
                        wgpu::BindGroupEntry {
                            binding: 8,
                            resource: wgpu::BindingResource::TextureView(
                                normal_texture.map_or(
                                    graphics
                                        .cache
                                        .texture_cache
                                        .empty_rgba16_float(&graphics)
                                        .wgpu_texture_view(),
                                    |x| unsafe { x.borrow_unsafe().0.wgpu_texture_view() },
                                ),
                            ),
                        },
                        // Normal sampler
                        wgpu::BindGroupEntry {
                            binding: 9,
                            resource: wgpu::BindingResource::Sampler(
                                normal_texture.map_or(
                                    graphics
                                        .cache
                                        .texture_cache
                                        .empty_rgba16_float(&graphics)
                                        .wgpu_sampler(),
                                    |x| unsafe { x.borrow_unsafe().0.wgpu_sampler() },
                                ),
                            ),
                        },
                        // Metallic type
                        wgpu::BindGroupEntry {
                            binding: 10,
                            resource: wgpu::BindingResource::Buffer(
                                metallic_type_buffer.as_entire_buffer_binding(),
                            ),
                        },
                        // Metallic value
                        wgpu::BindGroupEntry {
                            binding: 11,
                            resource: wgpu::BindingResource::Buffer(
                                metallic_value_buffer.as_entire_buffer_binding(),
                            ),
                        },
                        // Metallic texture
                        wgpu::BindGroupEntry {
                            binding: 12,
                            resource: wgpu::BindingResource::TextureView(
                                metallic_texture.map_or(
                                    graphics
                                        .cache
                                        .texture_cache
                                        .empty_r8_unorm(&graphics)
                                        .wgpu_texture_view(),
                                    |x| unsafe { x.borrow_unsafe().0.wgpu_texture_view() },
                                ),
                            ),
                        },
                        // Metallic sampler
                        wgpu::BindGroupEntry {
                            binding: 13,
                            resource: wgpu::BindingResource::Sampler(
                                metallic_texture.map_or(
                                    graphics
                                        .cache
                                        .texture_cache
                                        .empty_r8_unorm(&graphics)
                                        .wgpu_sampler(),
                                    |x| unsafe { x.borrow_unsafe().0.wgpu_sampler() },
                                ),
                            ),
                        },
                        // Roughness type
                        wgpu::BindGroupEntry {
                            binding: 14,
                            resource: wgpu::BindingResource::Buffer(
                                roughness_type_buffer.as_entire_buffer_binding(),
                            ),
                        },
                        // Roughness value
                        wgpu::BindGroupEntry {
                            binding: 15,
                            resource: wgpu::BindingResource::Buffer(
                                roughness_value_buffer.as_entire_buffer_binding(),
                            ),
                        },
                        // Roughness texture
                        wgpu::BindGroupEntry {
                            binding: 16,
                            resource: wgpu::BindingResource::TextureView(
                                roughness_texture.map_or(
                                    graphics
                                        .cache
                                        .texture_cache
                                        .empty_r8_unorm(&graphics)
                                        .wgpu_texture_view(),
                                    |x| unsafe { x.borrow_unsafe().0.wgpu_texture_view() },
                                ),
                            ),
                        },
                        // Roughness sampler
                        wgpu::BindGroupEntry {
                            binding: 17,
                            resource: wgpu::BindingResource::Sampler(
                                roughness_texture.map_or(
                                    graphics
                                        .cache
                                        .texture_cache
                                        .empty_r8_unorm(&graphics)
                                        .wgpu_sampler(),
                                    |x| unsafe { x.borrow_unsafe().0.wgpu_sampler() },
                                ),
                            ),
                        },
                    ],
                }),
        )
    }

    /// Create a new material. Check the material struct definition for more information.
    pub fn new(
        graphics: &Arc<RwLock<Graphics>>,
        unlit: bool,
        wireframe: Option<[f32; 4]>,
        albedo: (
            Option<[f32; 4]>,
            Option<Asset<TextureAsset<{ TextureType::RGBA8UnormSrgb }>>>,
        ),
        normal: Option<Asset<TextureAsset<{ TextureType::RGBA16Float }>>>,
        metallic: Either<f32, Asset<TextureAsset<{ TextureType::R8Unorm }>>>,
        roughness: Either<f32, Asset<TextureAsset<{ TextureType::R8Unorm }>>>,
    ) -> Self {
        let mut m = Material {
            id: MATERIAL_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            unlit,
            wireframe,
            albedo,
            normal,
            metallic,
            roughness,
            bind_group: None,
            graphics_weak_ref: Arc::downgrade(graphics),
        };

        m.generate_bind_group();

        m
    }

    pub fn id(&self) -> &usize {
        &self.id
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group.as_ref().unwrap()
    }

    /// If unlit is true, the material will not be affected by lighting.
    pub fn set_unlit(&mut self, unlit: bool) {
        self.unlit = unlit;

        self.generate_bind_group();
    }

    /// If a color is set, the material will be rendered as a wireframe with that color.
    pub fn set_wireframe(&mut self, wireframe: Option<[f32; 4]>) {
        self.wireframe = wireframe;

        self.generate_bind_group();
    }

    /// The base color of the material.
    /// If only one of either the texture or color are set, it will be used.
    /// If both are set, the texture's color will be multiplied by the color.
    pub fn set_albedo(
        &mut self,
        color: Option<[f32; 4]>,
        texture: Option<Asset<TextureAsset<{ Self::ALBEDO_TEXTURE_TYPE }>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if color.is_none() && texture.is_none() {
            return Err("Both color and texture cannot be None.".into());
        }

        self.albedo = (color, texture);

        self.generate_bind_group();

        Ok(())
    }

    /// Normal map, adds bumrps and details to the surface.
    pub fn set_normal(&mut self, normal: Option<Asset<TextureAsset<{ Self::NORMAL_TEXTURE_TYPE }>>>) {
        self.normal = normal;

        self.generate_bind_group();
    }

    /// Metallic map or value.
    pub fn set_metallic(
        &mut self,
        metallic: Either<f32, Asset<TextureAsset<{ Self::METALLIC_TEXTURE_TYPE }>>>,
    ) {
        self.metallic = metallic;

        self.generate_bind_group();
    }

    /// Roughness map or value.
    pub fn set_roughness(
        &mut self,
        roughness: Either<f32, Asset<TextureAsset<{ Self::ROUGHNESS_TEXTURE_TYPE }>>>,
    ) {
        self.roughness = roughness;

        self.generate_bind_group();
    }

    pub fn unlit(&self) -> &bool {
        &self.unlit
    }

    pub fn wireframe(&self) -> &Option<[f32; 4]> {
        &self.wireframe
    }

    pub fn albedo(
        &self,
    ) -> &(
        Option<[f32; 4]>,
        Option<Asset<TextureAsset<{ Self::ALBEDO_TEXTURE_TYPE }>>>,
    ) {
        &self.albedo
    }

    pub fn normal(&self) -> &Option<Asset<TextureAsset<{ Self::NORMAL_TEXTURE_TYPE }>>> {
        &self.normal
    }

    pub fn metallic(&self) -> &Either<f32, Asset<TextureAsset<{ Self::METALLIC_TEXTURE_TYPE }>>> {
        &self.metallic
    }

    pub fn roughness(&self) -> &Either<f32, Asset<TextureAsset<{ Self::ROUGHNESS_TEXTURE_TYPE }>>> {
        &self.roughness
    }
}

// Material bind group layout.
// 0. Unlit [bool]
// 1. Wireframe [bool]
// 2. WireframeColor [vec4]
// 3. AlbedoSupplied [u32 (BindingSuppliedType)] (1 = color, 2 = texture, 3 = both)
// 4. AlbedoColor [vec4]
// 5. AlbedoTexture [buffer]
// 6. AlbedoSampler [sampler]
// 7. NormalSupplied [bool]
// 8. NormalTexture [buffer]
// 9. NormalSampler [sampler]
// 10. MetallicType [u32 (BindingSuppliedType)] (1 = value, 2 = texture)
// 11. MetallicValue [f32]
// 12. MetallicTexture [buffer]
// 13. MetallicSampler [sampler]
// 14. RoughnessType [u32 (BindingSuppliedType)] (1 = value, 2 = texture)
// 15. RoughnessValue [f32]
// 16. RoughnessTexture [buffer]
// 17. RoughnessSampler [sampler]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub(crate) enum BindingSuppliedType {
    Value = 1,
    Texture = 2,
    Both = 3,
}

// TODO: Can these be storage textures?
fn create_material_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Material Bind Group Layout"),
        entries: &[
            // Unlit [bool]
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Wireframe [bool]
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // WireframeColor [vec4]
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // AlbedoSupplied [u32 (BindingSuppliedType)]
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // AlbedoColor [vec4]
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // AlbedoTexture [buffer]
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // AlbedoSampler [sampler]
            wgpu::BindGroupLayoutEntry {
                binding: 6,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
            // NormalSupplied [bool]
            wgpu::BindGroupLayoutEntry {
                binding: 7,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // NormalTexture [buffer]
            wgpu::BindGroupLayoutEntry {
                binding: 8,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // NormalSampler [sampler]
            wgpu::BindGroupLayoutEntry {
                binding: 9,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
            // MetallicType [u32 (BindingSuppliedType)]
            wgpu::BindGroupLayoutEntry {
                binding: 10,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // MetallicValue [f32]
            wgpu::BindGroupLayoutEntry {
                binding: 11,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // MetallicTexture [buffer]
            wgpu::BindGroupLayoutEntry {
                binding: 12,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // MetallicSampler [sampler]
            wgpu::BindGroupLayoutEntry {
                binding: 13,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
            // RoughnessType [u32 (BindingSuppliedType)]
            wgpu::BindGroupLayoutEntry {
                binding: 14,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // RoughnessValue [f32]
            wgpu::BindGroupLayoutEntry {
                binding: 15,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // RoughnessTexture [buffer]
            wgpu::BindGroupLayoutEntry {
                binding: 16,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // RoughnessSampler [sampler]
            wgpu::BindGroupLayoutEntry {
                binding: 17,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
        ],
    })
}

impl HasBindGroupLayout<()> for Material {
    fn bind_group_layout<'a>(graphics: &'a Graphics, _extra: ()) -> &'a wgpu::BindGroupLayout {
        &graphics
            .cache
            .bind_group_layout_cache
            .material
            .get_or_init(|| create_material_bind_group_layout(&graphics.device))
    }
}

impl std::cmp::PartialEq for Material {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl std::cmp::Eq for Material {}

impl std::cmp::PartialOrd for Material {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for Material {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl Material {
    /// Default instance of `Material`.
    /// White matte material.
    pub fn default(graphics: &Arc<RwLock<Graphics>>) -> Self {
        let mut m = Material {
            id: MATERIAL_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            unlit: false,
            wireframe: None,
            albedo: (Some([1.0, 1.0, 1.0, 1.0]), None),
            normal: None,
            roughness: Either::Left(0.5),
            metallic: Either::Left(0.0),
            bind_group: None,
            graphics_weak_ref: Arc::downgrade(graphics),
        };

        m.generate_bind_group();

        m
    }
}

impl ResourceTrait for Material {}

impl AssetTrait for Material {
    fn type_name() -> String {
        "Material".to_owned()
    }

    fn fs_type() -> AssetFileSystemType {
        AssetFileSystemType::File
    }

    fn read_packed_buffer(
        data: &mut dyn std::io::Read,
        graphics: &Graphics,
    ) -> Result<Self, cobalt_assets::server::AssetLoadError> {
        todo!()
    }

    fn read_source_file_to_buffer(
        abs_path: &std::path::Path,
    ) -> Result<Bytes, cobalt_assets::server::AssetLoadError> {
        todo!()
    }

    fn read_source_file(
        abs_path: &std::path::Path,
        graphics: &Graphics,
    ) -> Result<Self, cobalt_assets::server::AssetLoadError> {
        todo!()
    }

    fn verify_source_file(
        abs_path: &std::path::Path,
    ) -> Result<(), cobalt_assets::server::AssetLoadError> {
        todo!()
    }
}
