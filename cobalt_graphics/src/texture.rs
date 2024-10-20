use half::f16;
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};
use std::{
    marker::ConstParamTy,
    sync::{LazyLock, OnceLock},
};

use crate::HasBindGroupLayout;

use super::{cache::TextureCache, context::Graphics, HasBindGroup};

// NOTE: If adding variants to this, change the `gen_empty_texture` function and cobalt_pack
#[derive(
    Debug, Clone, Copy, PartialEq, ConstParamTy, Eq, serde::Serialize, serde::Deserialize, Hash,
)]
pub enum TextureType {
    // Color textures
    RGBA32Float,
    RGBA16Float,
    RGBA8Unorm,
    /// SRGB texture, used for color textures such as albedo.
    RGBA8UnormSrgb,

    // Gray scale textures
    R32Float,
    R16Float,
    R8Unorm,
    R8Uint,
    R8Snorm,
}

impl TextureType {
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            TextureType::RGBA32Float => 16,
            TextureType::RGBA16Float => 8,
            TextureType::RGBA8Unorm => 4,
            TextureType::RGBA8UnormSrgb => 4,

            TextureType::R32Float => 4,
            TextureType::R16Float => 2,
            TextureType::R8Unorm => 1,
            TextureType::R8Uint => 1,
            TextureType::R8Snorm => 1,
        }
    }

    pub fn variants() -> Vec<Self> {
        vec![
            TextureType::RGBA32Float,
            TextureType::RGBA16Float,
            TextureType::RGBA8Unorm,
            TextureType::RGBA8UnormSrgb,
            TextureType::R32Float,
            TextureType::R16Float,
            TextureType::R8Unorm,
            TextureType::R8Uint,
            TextureType::R8Snorm,
        ]
    }

    pub fn to_string(&self) -> String {
        match self {
            TextureType::RGBA32Float => "RGBA32Float".to_string(),
            TextureType::RGBA16Float => "RGBA16Float".to_string(),
            TextureType::RGBA8Unorm => "RGBA8Unorm".to_string(),
            TextureType::RGBA8UnormSrgb => "RGBA8UnormSrgb".to_string(),

            TextureType::R32Float => "R32Float".to_string(),
            TextureType::R16Float => "R16Float".to_string(),
            TextureType::R8Unorm => "R8Unorm".to_string(),
            TextureType::R8Uint => "R8Uint".to_string(),
            TextureType::R8Snorm => "R8Snorm".to_string(),
        }
    }

    // Tries to get image data from a dynamic image.
    pub fn get_image_data(&self, image: image::DynamicImage) -> Result<bytes::Bytes, Box<dyn std::error::Error>> {
        let vec_res: Result<Vec<u8>, String> = match self {
            TextureType::RGBA8Unorm => Ok(image.into_rgba8().into_vec()),
            TextureType::RGBA8UnormSrgb => Ok(image.into_rgba8().into_vec()),
            TextureType::RGBA32Float => Ok(bytemuck::cast_vec(image.into_rgba32f().into_vec())),
            TextureType::RGBA16Float => Ok({
                let image_data = image
                    .into_rgba32f()
                    .into_vec()
                    .iter()
                    .map(|f| f16::from_f32(*f))
                    .collect::<Vec<f16>>();

                let bytes: &[u8] = bytemuck::must_cast_slice(&image_data);

                bytes.to_vec()
            }),

            TextureType::R32Float => Ok(bytemuck::cast_vec(
                image
                    .into_luma16()
                    .iter()
                    .map(|u| *u as f32)
                    .collect::<Vec<f32>>(),
            )),
            TextureType::R16Float => Ok(bytemuck::cast_vec({
                let image_data = image
                    .into_luma16()
                    .iter()
                    .map(|u| f16::from_f32(*u as f32))
                    .collect::<Vec<f16>>();

                let bytes: &[u8] = bytemuck::must_cast_slice(&image_data);

                bytes.to_vec()
            })),
            TextureType::R8Unorm => Ok(image.into_luma8().into_vec()),
            TextureType::R8Uint => Ok(image.into_luma8().into_vec()),
            TextureType::R8Snorm => Ok(bytemuck::cast_vec(
                image
                    .into_luma8()
                    .iter()
                    .map(|u| *u as i8)
                    .collect::<Vec<i8>>(),
            )),
        };

        Ok(bytes::Bytes::from(vec_res?))
    }

    pub fn buffer_to_dyn_image(
        &self,
        buffer: Vec<u8>,
        width: u32,
        height: u32,
    ) -> image::DynamicImage {
        match self {
            TextureType::RGBA8Unorm => image::DynamicImage::ImageRgba8(
                image::RgbaImage::from_raw(width, height, buffer).unwrap(),
            ),
            TextureType::RGBA8UnormSrgb => image::DynamicImage::ImageRgba8(
                image::RgbaImage::from_raw(width, height, buffer).unwrap(),
            ),
            TextureType::RGBA32Float => image::DynamicImage::ImageRgba32F(
                image::Rgba32FImage::from_raw(width, height, bytemuck::cast_vec(buffer)).unwrap(),
            ),
            TextureType::RGBA16Float => image::DynamicImage::ImageRgba32F(
                image::Rgba32FImage::from_raw(width, height, bytemuck::cast_vec(buffer)).unwrap(),
            ),
            TextureType::R32Float => image::DynamicImage::ImageLuma16(
                image::ImageBuffer::from_raw(width, height, bytemuck::cast_vec(buffer)).unwrap(),
            ),
            TextureType::R16Float => image::DynamicImage::ImageLuma16(
                image::ImageBuffer::from_raw(width, height, bytemuck::cast_vec(buffer)).unwrap(),
            ),
            TextureType::R8Unorm => image::DynamicImage::ImageLuma8(
                image::ImageBuffer::from_raw(width, height, buffer).unwrap(),
            ),
            TextureType::R8Uint => image::DynamicImage::ImageLuma8(
                image::ImageBuffer::from_raw(width, height, buffer).unwrap(),
            ),
            TextureType::R8Snorm => image::DynamicImage::ImageLuma8(
                image::ImageBuffer::from_raw(width, height, buffer).unwrap(),
            ),
        }
    }
}

impl Into<wgpu::TextureFormat> for TextureType {
    fn into(self) -> wgpu::TextureFormat {
        match self {
            TextureType::RGBA32Float => wgpu::TextureFormat::Rgba32Float,
            TextureType::RGBA16Float => wgpu::TextureFormat::Rgba16Float,
            TextureType::RGBA8Unorm => wgpu::TextureFormat::Rgba8Unorm,
            TextureType::RGBA8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,

            TextureType::R32Float => wgpu::TextureFormat::R32Float,
            TextureType::R16Float => wgpu::TextureFormat::R16Float,
            TextureType::R8Unorm => wgpu::TextureFormat::R8Unorm,
            TextureType::R8Uint => wgpu::TextureFormat::R8Uint,
            TextureType::R8Snorm => wgpu::TextureFormat::R8Snorm,
        }
    }
}

impl std::fmt::Display for TextureType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Debug)]
pub enum GenericTexture {
    RGBA32Float(Texture<{ TextureType::RGBA32Float }>),
    RGBA16Float(Texture<{ TextureType::RGBA16Float }>),
    RGBA8Unorm(Texture<{ TextureType::RGBA8Unorm }>),
    RGBA8UnormSrgb(Texture<{ TextureType::RGBA8UnormSrgb }>),
    R32Float(Texture<{ TextureType::R32Float }>),
    R16Float(Texture<{ TextureType::R16Float }>),
    R8Unorm(Texture<{ TextureType::R8Unorm }>),
    R8Uint(Texture<{ TextureType::R8Uint }>),
    R8Snorm(Texture<{ TextureType::R8Snorm }>),
}

pub struct Texture<const T: TextureType> {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub size: wgpu::Extent3d,
    // TODO: Bind group dirty after changing texture?
    pub bind_group: wgpu::BindGroup,
}

impl<const T: TextureType> std::fmt::Debug for Texture<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Texture")
            .field("size", &self.size)
            .field("format", &T)
            .finish()
    }
}

pub trait TextureInternal {
    fn size(&self) -> &wgpu::Extent3d;

    fn size_mut(&mut self) -> &mut wgpu::Extent3d;

    fn wgpu_texture(&self) -> &wgpu::Texture;

    fn wgpu_texture_mut(&mut self) -> &mut wgpu::Texture;

    fn wgpu_texture_view(&self) -> &wgpu::TextureView;

    fn wgpu_texture_view_mut(&mut self) -> &mut wgpu::TextureView;

    fn wgpu_sampler(&self) -> &wgpu::Sampler;

    fn wgpu_sampler_mut(&mut self) -> &mut wgpu::Sampler;
}

impl<const T: TextureType> TextureInternal for Texture<T> {
    fn size(&self) -> &wgpu::Extent3d {
        &self.size
    }

    fn size_mut(&mut self) -> &mut wgpu::Extent3d {
        &mut self.size
    }

    fn wgpu_texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    fn wgpu_texture_mut(&mut self) -> &mut wgpu::Texture {
        &mut self.texture
    }

    fn wgpu_texture_view(&self) -> &wgpu::TextureView {
        &self.view
    }

    fn wgpu_texture_view_mut(&mut self) -> &mut wgpu::TextureView {
        &mut self.view
    }

    fn wgpu_sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    fn wgpu_sampler_mut(&mut self) -> &mut wgpu::Sampler {
        &mut self.sampler
    }
}

impl<const T: TextureType> Texture<T> {
    pub fn size(&self) -> (u32, u32) {
        (self.size.width, self.size.height)
    }
}

// static TEXTURE_BIND_GROUP_LAYOUT_FILTERING_FILTERABLE: LazyLock<wgpu::BindGroupLayout> =
//     LazyLock::new(|| create_bind_group_layout(true, true));

// static TEXTURE_BIND_GROUP_LAYOUT_NON_FILTERING_NON_FILTERABLE: LazyLock<wgpu::BindGroupLayout> =
//     LazyLock::new(|| create_bind_group_layout(false, false));

fn create_bind_group_layout(
    device: &wgpu::Device,
    filterable: bool,
    filtering: bool,
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(match filtering {
                    true => wgpu::SamplerBindingType::Filtering,
                    false => wgpu::SamplerBindingType::NonFiltering,
                }),
                count: None,
            },
        ],
    })
}

// fn get_bind_group_layout(ty: TextureType) -> MappedRwLockReadGuard<wgpu::BindGroupLayout> {
//     match ty {
//         TextureType::RGBA32Float
//         | TextureType::RGBA16Float
//         | TextureType::RGBA8Unorm
//         | TextureType::RGBA8UnormSrgb => &TEXTURE_BIND_GROUP_LAYOUT_FILTERING_FILTERABLE,
//         TextureType::R32Float
//         | TextureType::R16Float
//         | TextureType::R8Unorm
//         | TextureType::R8Uint
//         | TextureType::R8Snorm => &TEXTURE_BIND_GROUP_LAYOUT_NON_FILTERING_NON_FILTERABLE,
//     }
// }

impl<const T: TextureType> HasBindGroup for Texture<T> {
    // TODO: Handle texture changes
    fn bind_group(&mut self, _: &Graphics) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

/// Create or retrieve a cached bind group layout for a texture type.
fn texture_bind_group_layout<'a, const T: TextureType>(
    graphics: &'a Graphics,
) -> &'a wgpu::BindGroupLayout {
    #[rustfmt::skip]
    let layout_cache_ref = match T {
        TextureType::RGBA32Float => &graphics.cache.bind_group_layout_cache.textures.rgba32_float,
        TextureType::RGBA16Float => &graphics.cache.bind_group_layout_cache.textures.rgba16_float,
        TextureType::RGBA8Unorm => &graphics.cache.bind_group_layout_cache.textures.rgba8_unorm,
        TextureType::RGBA8UnormSrgb => &graphics.cache.bind_group_layout_cache.textures.rgba8_unorm_srgb,
        TextureType::R32Float => &graphics.cache.bind_group_layout_cache.textures.r32_float,
        TextureType::R16Float => &graphics.cache.bind_group_layout_cache.textures.r16_float,
        TextureType::R8Unorm => &graphics.cache.bind_group_layout_cache.textures.r8_unorm,
        TextureType::R8Uint => &graphics.cache.bind_group_layout_cache.textures.r8_uint,
        TextureType::R8Snorm => &graphics.cache.bind_group_layout_cache.textures.r8_snorm,
    };

    layout_cache_ref.get_or_init(|| {
        let filterable = match T {
            TextureType::RGBA32Float
            | TextureType::RGBA16Float
            | TextureType::RGBA8Unorm
            | TextureType::RGBA8UnormSrgb => true,
            TextureType::R32Float
            | TextureType::R16Float
            | TextureType::R8Unorm
            | TextureType::R8Uint
            | TextureType::R8Snorm => false,
        };

        let filtering = match T {
            TextureType::RGBA32Float
            | TextureType::RGBA16Float
            | TextureType::RGBA8Unorm
            | TextureType::RGBA8UnormSrgb => true,
            TextureType::R32Float
            | TextureType::R16Float
            | TextureType::R8Unorm
            | TextureType::R8Uint
            | TextureType::R8Snorm => false,
        };

        create_bind_group_layout(&graphics.device, filterable, filtering)
    })
}

impl<const T: TextureType> HasBindGroupLayout<()> for Texture<T> {
    fn bind_group_layout<'a>(graphics: &'a Graphics, extra: ()) -> &'a wgpu::BindGroupLayout {
        texture_bind_group_layout::<T>(graphics)
    }
}

pub(crate) fn gen_empty_texture<const T: TextureType>(graphics: &Graphics) -> Texture<T> {
    let size = wgpu::Extent3d {
        width: 1,
        height: 1,
        depth_or_array_layers: 1,
    };

    let texture = graphics.device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: T.into(),
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[T.into()],
    });

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let sampler = graphics.device.create_sampler(&wgpu::SamplerDescriptor {
        label: None,
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let bind_group = graphics
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &Texture::<{ T }>::bind_group_layout(graphics, ()),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

    fn write_texture(
        graphics: &Graphics,
        data: &[u8],
        bytes_per_row: u32,
        texture: &wgpu::Texture,
        size: wgpu::Extent3d,
    ) {
        graphics.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(size.height),
            },
            size,
        );
    }

    match T {
        TextureType::RGBA32Float => {
            write_texture(
                graphics,
                bytemuck::cast_slice(&[1.0f32, 1.0, 1.0, 1.0]),
                16,
                &texture,
                size,
            );
        }
        TextureType::RGBA16Float => {
            write_texture(
                graphics,
                bytemuck::cast_slice(&[
                    f16::from_f32(1.0),
                    f16::from_f32(1.0),
                    f16::from_f32(1.0),
                    f16::from_f32(1.0),
                ]),
                8,
                &texture,
                size,
            );
        }
        TextureType::RGBA8Unorm => {
            write_texture(graphics, &[255u8, 255, 255, 255], 4, &texture, size);
        }
        TextureType::RGBA8UnormSrgb => {
            write_texture(graphics, &[255u8, 255, 255, 255], 4, &texture, size);
        }
        TextureType::R32Float => {
            write_texture(graphics, bytemuck::cast_slice(&[1.0f32]), 4, &texture, size);
        }
        TextureType::R16Float => {
            write_texture(
                graphics,
                bytemuck::cast_slice(&[f16::from_f32(1.0)]),
                2,
                &texture,
                size,
            );
        }
        TextureType::R8Unorm => {
            write_texture(graphics, &[255u8], 1, &texture, size);
        }
        TextureType::R8Uint => {
            write_texture(graphics, &[255u8], 1, &texture, size);
        }
        TextureType::R8Snorm => {
            write_texture(graphics, bytemuck::cast_slice(&[127i8]), 1, &texture, size);
        }
    }

    Texture {
        texture,
        view,
        sampler,
        size,
        bind_group,
    }
}

static EMPTY_RGBA32_FLOAT_TEXTURE: LazyLock<RwLock<Option<Texture<{ TextureType::RGBA32Float }>>>> =
    LazyLock::new(|| RwLock::new(None));

pub fn empty_rgba32_float_texture(
    graphics: &Graphics,
) -> MappedRwLockReadGuard<'static, Texture<{ TextureType::RGBA32Float }>> {
    if EMPTY_RGBA32_FLOAT_TEXTURE.read().is_none() {
        EMPTY_RGBA32_FLOAT_TEXTURE
            .write()
            .replace(gen_empty_texture(graphics));
    }

    RwLockReadGuard::map(EMPTY_RGBA32_FLOAT_TEXTURE.read(), |opt| {
        opt.as_ref().unwrap()
    })
}

impl TextureCache {
    pub fn new() -> Self {
        Self {
            empty_rgba32_float: OnceLock::new(),
            empty_rgba16_float: OnceLock::new(),
            empty_rgba8_unorm: OnceLock::new(),
            empty_rgba8_unorm_srgb: OnceLock::new(),
            empty_r32_float: OnceLock::new(),
            empty_r16_float: OnceLock::new(),
            empty_r8_unorm: OnceLock::new(),
            empty_r8_uint: OnceLock::new(),
            empty_r8_snorm: OnceLock::new(),
        }
    }

    pub fn empty_rgba32_float<'a>(
        &'a self,
        graphics: &Graphics,
    ) -> &'a Texture<{ TextureType::RGBA32Float }> {
        self.empty_rgba32_float
            .get_or_init(|| gen_empty_texture(graphics))
    }

    pub fn empty_rgba16_float<'a>(
        &'a self,
        graphics: &Graphics,
    ) -> &'a Texture<{ TextureType::RGBA16Float }> {
        self.empty_rgba16_float
            .get_or_init(|| gen_empty_texture(graphics))
    }

    pub fn empty_rgba8_unorm<'a>(
        &'a self,
        graphics: &Graphics,
    ) -> &'a Texture<{ TextureType::RGBA8Unorm }> {
        self.empty_rgba8_unorm
            .get_or_init(|| gen_empty_texture(graphics))
    }

    pub fn empty_rgba8_unorm_srgb<'a>(
        &'a self,
        graphics: &Graphics,
    ) -> &'a Texture<{ TextureType::RGBA8UnormSrgb }> {
        self.empty_rgba8_unorm_srgb
            .get_or_init(|| gen_empty_texture(graphics))
    }

    pub fn empty_r32_float<'a>(
        &'a self,
        graphics: &Graphics,
    ) -> &'a Texture<{ TextureType::R32Float }> {
        self.empty_r32_float
            .get_or_init(|| gen_empty_texture(graphics))
    }

    pub fn empty_r16_float<'a>(
        &'a self,
        graphics: &Graphics,
    ) -> &'a Texture<{ TextureType::R16Float }> {
        self.empty_r16_float
            .get_or_init(|| gen_empty_texture(graphics))
    }

    pub fn empty_r8_unorm<'a>(
        &'a self,
        graphics: &Graphics,
    ) -> &'a Texture<{ TextureType::R8Unorm }> {
        self.empty_r8_unorm
            .get_or_init(|| gen_empty_texture(graphics))
    }

    pub fn empty_r8_uint<'a>(
        &'a self,
        graphics: &Graphics,
    ) -> &'a Texture<{ TextureType::R8Uint }> {
        self.empty_r8_uint
            .get_or_init(|| gen_empty_texture(graphics))
    }

    pub fn empty_r8_snorm<'a>(
        &'a self,
        graphics: &Graphics,
    ) -> &'a Texture<{ TextureType::R8Snorm }> {
        self.empty_r8_snorm
            .get_or_init(|| gen_empty_texture(graphics))
    }
}
