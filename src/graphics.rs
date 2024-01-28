use pollster::FutureExt;
use wgpu::SurfaceTargetUnsafe;

pub struct Frame<'a> {
    encoder: wgpu::CommandEncoder,
    swap_texture: wgpu::SurfaceTexture,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl Frame<'_> {
    pub fn encoder(&mut self) -> &mut wgpu::CommandEncoder {
        &mut self.encoder
    }

    pub fn swap_texture(&self) -> &wgpu::SurfaceTexture {
        &self.swap_texture
    }

    pub fn clear(&mut self, color: wgpu::Color) {
        let view = self.swap_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let _render_pass = self
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
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

pub struct Graphics {
    pub(crate) instance: wgpu::Instance,
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) surface: wgpu::Surface<'static>,
}

impl Graphics {
    pub(crate) fn new(
        window: &Window,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let instance = wgpu::Instance::default();

        let surface =
            unsafe { instance.create_surface_unsafe(SurfaceTargetUnsafe::from_window(&window.winit)?) }
                .map_err(|_| GraphicsError::CreateSurfaceError)?;

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
                    required_features: wgpu::Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .block_on()
            .map_err(|_| GraphicsError::DeviceError)?;
        
        let mut res = Self {
            instance,
            adapter,
            device,
            surface,
            queue,
        };
        
        res.configure_surface(window.winit.inner_size().into());

        Ok(res)
    }

    pub(crate) fn configure_surface(&self, size: (u32, u32)) {
        let surface_capabilities = self.surface.get_capabilities(&self.adapter);

        // Get preferred format
        let format = surface_capabilities.formats[0];

        self.surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                width: size.0,
                height: size.1,
                present_mode: wgpu::PresentMode::AutoNoVsync,
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![wgpu::TextureFormat::Bgra8UnormSrgb],
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

        Ok(Self {
            winit: window,
        })
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
