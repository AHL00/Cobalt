use std::sync::LazyLock;
use wgpu::util::DeviceExt;

pub mod context;
pub mod frame;
pub mod texture;
pub mod vertex;
pub mod window;

pub use winit;
pub use wgpu;

pub mod exports {
    pub use super::wgpu;
    pub use super::window;
}

/// Capable of creating a wgpu::BindGroupLayout.
pub trait HasBindGroupLayout {
    fn bind_group_layout() -> &'static wgpu::BindGroupLayout;
}

pub trait HasBindGroup {
    /// Returns a reference to the bind group.
    /// Needs to be mutable because the bind group might be dirty and need to be recreated.
    /// References graphics() to get the device.
    fn bind_group(&mut self, graphics: &self::context::Graphics) -> &wgpu::BindGroup;
}

pub trait HasStableBindGroup {
    /// Returns a reference to the bind group.
    /// The bind group is guaranteed to be stable and not need to be recreated.
    fn stable_bind_group(&self) -> &wgpu::BindGroup;
}

pub trait CreateBindGroup {
    fn create_bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup;
}

static MAT4X4_BIND_GROUP_LAYOUT: LazyLock<wgpu::BindGroupLayout> = LazyLock::new(|| {
    self::context::Graphics::global_read()
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
});

impl CreateBindGroup for ultraviolet::Mat4 {
    fn create_bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(self.as_byte_slice()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &*MAT4X4_BIND_GROUP_LAYOUT,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
            }],
        })
    }
}

impl HasBindGroupLayout for ultraviolet::Mat4 {
    fn bind_group_layout() -> &'static wgpu::BindGroupLayout {
        &*MAT4X4_BIND_GROUP_LAYOUT
    }
}

static U32_BIND_GROUP_LAYOUT: LazyLock<wgpu::BindGroupLayout> = LazyLock::new(|| {
    self::context::Graphics::global_read()
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
});

impl CreateBindGroup for u32 {
    fn create_bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[*self]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &*U32_BIND_GROUP_LAYOUT,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
            }],
        })
    }
}

impl HasBindGroupLayout for u32 {
    fn bind_group_layout() -> &'static wgpu::BindGroupLayout {
        &*U32_BIND_GROUP_LAYOUT
    }
}

pub trait HasVertexBufferLayout {
    fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static>;
}

#[derive(thiserror::Error, Debug)]
pub enum GraphicsError {
    #[error("Failed to request adapter")]
    NoAdapter,
    #[error("Failed to create surface")]
    CreateSurfaceError,
    #[error("Failed to create device")]
    DeviceError,
    #[error("Failed to create swap chain")]
    SwapChainError,
    #[error("Failed to get next swap chain texture")]
    SwapChainTextureError,
}
