use std::sync::LazyLock;

use pollster::FutureExt;
use wgpu::{util::DeviceExt, SurfaceTargetUnsafe};

use crate::engine::graphics;

pub mod texture;
pub mod vertex;

pub use winit::window as winit_window;

pub struct Frame<'a> {
    encoder: wgpu::CommandEncoder,
    swap_texture: wgpu::SurfaceTexture,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl Frame<'_> {
    /// TODO: For multithreading, we could make new encoders every time this is called.
    /// Store the encoders in a vec and then submit them all at the end of the frame.
    pub fn encoder(&mut self) -> &mut wgpu::CommandEncoder {
        &mut self.encoder
    }

    pub fn swap_texture(&self) -> &wgpu::SurfaceTexture {
        &self.swap_texture
    }

    pub fn clear(&mut self, color: wgpu::Color) {
        let view = self
            .swap_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let _render_pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            timestamp_writes: None,
            occlusion_query_set: None,
            depth_stencil_attachment: None,
        });
    }
}

/// Capable of creating a wgpu::BindGroupLayout.
pub(crate) trait HasBindGroupLayout {
    fn bind_group_layout() -> &'static wgpu::BindGroupLayout;
}

pub(crate) trait HasBindGroup {
    /// Returns a reference to the bind group.
    /// Needs to be mutable because the bind group might be dirty and need to be recreated.
    /// References graphics() to get the device.
    fn bind_group(&mut self, graphics: &Graphics) -> &wgpu::BindGroup;
}

pub(crate) trait CreateBindGroup {
    fn create_bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup;
}

static MAT4X4_BIND_GROUP_LAYOUT: LazyLock<wgpu::BindGroupLayout> = LazyLock::new(|| {
    graphics()
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

pub(crate) trait HasVertexBufferLayout {
    fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static>;
}

pub struct Graphics {
    pub(crate) instance: wgpu::Instance,
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) output_color_format: wgpu::TextureFormat,
    pub(crate) output_depth_format: Option<wgpu::TextureFormat>,
}

impl Graphics {
    pub(crate) fn new(window: &Window) -> Result<Self, Box<dyn std::error::Error>> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(debug)]
            flags: wgpu::InstanceFlags::debugging(),
            #[cfg(not(debug))]
            flags: wgpu::InstanceFlags::empty(),
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = unsafe {
            instance.create_surface_unsafe(SurfaceTargetUnsafe::from_window(&window.winit)?)
        }
        .map_err(|e| {
            log::error!("Failed to create surface: {}", e);
            GraphicsError::CreateSurfaceError
        })?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .block_on()
            .ok_or(GraphicsError::NoAdapter)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    // TODO: Disable this feature when wireframe is not used, release builds
                    required_features: wgpu::Features::POLYGON_MODE_LINE,
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    required_limits: wgpu::Limits::downlevel_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .block_on()
            .map_err(|e| {
                log::error!("Failed to create device: {}", e);
                GraphicsError::DeviceError
            })?;

        let surface_capabilities = surface.get_capabilities(&adapter);

        // Get preferred format
        let output_color_format = if surface_capabilities.formats.is_empty() {
            log::warn!("No preferred format found, using Bgra8UnormSrgb");
            wgpu::TextureFormat::Bgra8UnormSrgb
        } else {
            surface_capabilities.formats[0]
        };

        let output_depth_format = Some(wgpu::TextureFormat::Depth32Float);

        let res = Self {
            instance,
            adapter,
            device,
            surface,
            queue,
            output_color_format,
            output_depth_format,
        };

        res.configure_surface(window.winit.inner_size().into());

        Ok(res)
    }

    pub(crate) fn configure_surface(&self, size: (u32, u32)) {
        self.surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.output_color_format,
                width: size.0,
                height: size.1,
                present_mode: wgpu::PresentMode::Mailbox,
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![self.output_color_format],
            },
        );
    }

    pub fn begin_frame<'a>(&self) -> Result<Frame<'a>, Box<dyn std::error::Error>> {
        let swap_texture = self
            .surface
            .get_current_texture()
            .map_err(|_| GraphicsError::SwapChainTextureError)?;

        let encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        Ok(Frame {
            encoder,
            swap_texture,
            _marker: std::marker::PhantomData,
        })
    }

    pub fn end_frame(&self, frame: Frame) {
        self.queue.submit(std::iter::once(frame.encoder.finish()));
        frame.swap_texture.present();
    }
}

impl Drop for Graphics {
    fn drop(&mut self) {
        self.device.poll(wgpu::Maintain::Wait);
    }
}

pub struct Window {
    // TODO: Maybe make this private and expose own methods?
    pub winit: winit::window::Window,
}

impl Window {
    pub(crate) fn new(
        event_loop: &winit::event_loop::EventLoop<()>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let window = winit::window::WindowBuilder::new()
            .with_title("Engine")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
            .build(event_loop)?;

        Ok(Self { winit: window })
    }
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
