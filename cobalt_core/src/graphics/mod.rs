pub mod context;
pub mod frame;
pub mod texture;
pub mod vertex;
pub mod window;
pub mod bind_groups;

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
