use std::sync::OnceLock;

use crate::graphics::context::Graphics;


pub const GEOMETRY_BUFFER_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;

pub struct GeometryBuffers {
    pub position_buffer: wgpu::Texture,
    pub normal_buffer: wgpu::Texture,
    /// Albedo and specular are packed into the same buffer.
    /// The first three channels are the albedo color, and the fourth channel is the specular intensity.
    pub albedo_specular_buffer: wgpu::Texture,

    pub bind_group: wgpu::BindGroup,
}

static BIND_GROUP_LAYOUT: OnceLock<wgpu::BindGroupLayout> = OnceLock::new();

impl GeometryBuffers {
    // TODO: Pack position and normal into the same buffer. First channel in normal to third
    // channel in position. Then normal will only be a Rg16Float.
    pub const POSITION_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
    pub const NORMAL_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
    pub const ALBEDO_SPECULAR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

    pub fn generate(size: (u32, u32)) -> Self {
            let graphics = Graphics::global_read();

            let position_buffer = graphics.device.create_texture(&wgpu::TextureDescriptor { 
                label: Some("Position Buffer"),
                size: wgpu::Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                },
                view_formats: &[Self::POSITION_FORMAT],
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: Self::POSITION_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,    
            });
            
            let normal_buffer = graphics.device.create_texture(&wgpu::TextureDescriptor { 
                label: Some("Normal Buffer"),
                size: wgpu::Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                },
                view_formats: &[Self::NORMAL_FORMAT],
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: Self::NORMAL_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,    
            });

            let albedo_specular_buffer = graphics.device.create_texture(&wgpu::TextureDescriptor { 
                label: Some("Albedo Specular Buffer"),
                size: wgpu::Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                },
                view_formats: &[Self::ALBEDO_SPECULAR_FORMAT],
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: Self::ALBEDO_SPECULAR_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,    
            });

            let bind_group_layout = BIND_GROUP_LAYOUT.get_or_init(|| {
                graphics.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Geometry Buffers Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                    ],
                })
            });

            let bind_group = graphics.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Geometry Buffers Bind Group"),
                layout: bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&position_buffer.create_view(&wgpu::TextureViewDescriptor::default())),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&normal_buffer.create_view(&wgpu::TextureViewDescriptor::default())),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&albedo_specular_buffer.create_view(&wgpu::TextureViewDescriptor::default())),
                    },
                ],
            });

            Self {
                position_buffer,
                normal_buffer,
                albedo_specular_buffer,

                bind_group,
            }
    }
}