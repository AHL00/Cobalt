use std::any::TypeId;

use parking_lot::{MappedRwLockReadGuard, RwLockReadGuard};
use pollster::FutureExt;
use wgpu::{PresentMode, SurfaceTargetUnsafe};

use crate::graphics::GraphicsError;

use super::{cache::GraphicsCache, exports::window::WindowInternal, frame::Frame, window::Window};

/// A global graphics context that manages the window, device, and other wgpu resources.
/// This is marked as public but not exported to the user, it should not be necessary to access this outside of the renderer.
pub struct Graphics {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub surface: wgpu::Surface<'static>,

    pub output_color_format: wgpu::TextureFormat,
    pub output_depth_format: Option<wgpu::TextureFormat>,
    pub current_present_mode: PresentMode,

    pub cache: GraphicsCache,
}

impl Graphics {
    /// Get a bind group layout from the cache, or create a new one if it doesn't exist.
    /// If a new layout is created, it is inserted into the cache.
    /// Layouts are unique to a type, so the type of the layout must be specified.
    /// The hashmap uses the layout's type id as the key.
    pub fn bind_group_layout_cache<'a, T: 'static>(
        &'a self,
        create_layout: impl FnOnce(&wgpu::Device) -> wgpu::BindGroupLayout,
    ) -> MappedRwLockReadGuard<'a, wgpu::BindGroupLayout> {
        let type_id = TypeId::of::<T>();

        if self
            .cache
            .bind_group_layout_cache
            .read()
            .contains_key(&type_id)
        {
            RwLockReadGuard::map(self.cache.bind_group_layout_cache.read(), |x| {
                x.get(&type_id).unwrap()
            })
        } else {
            let layout = create_layout(&self.device);
            self.cache
                .bind_group_layout_cache
                .write()
                .insert(type_id, layout);
            RwLockReadGuard::map(self.cache.bind_group_layout_cache.read(), |x| {
                x.get(&type_id).unwrap()
            })
        }
    }

    /// create_layout: Function to create the layout for `T` does not exist.
    /// This is required because of lifetime issues with borrowing the entire
    /// Graphics context if using `bind_group_layout` and the wgpu api directly.
    pub fn create_bind_group<T: 'static>(
        &self,
        label: Option<&str>,
        entries: &[wgpu::BindGroupEntry],
        create_layout: impl FnOnce(&wgpu::Device) -> wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        let type_id = TypeId::of::<T>();

        let layout = if self
            .cache
            .bind_group_layout_cache
            .read()
            .contains_key(&type_id)
        {
            RwLockReadGuard::map(self.cache.bind_group_layout_cache.read(), |x| {
                x.get(&type_id).unwrap()
            })
        } else {
            let layout = create_layout(&self.device);
            self.cache
                .bind_group_layout_cache
                .write()
                .insert(type_id, layout);
            RwLockReadGuard::map(self.cache.bind_group_layout_cache.read(), |x| {
                x.get(&type_id).unwrap()
            })
        };

        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries,
            label,
        })
    }

    pub fn new(window: &Window) -> Result<Self, Box<dyn std::error::Error>> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(debug_assertions)]
            flags: wgpu::InstanceFlags::debugging(),
            #[cfg(not(debug_assertions))]
            flags: wgpu::InstanceFlags::empty(),
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = unsafe {
            instance.create_surface_unsafe(SurfaceTargetUnsafe::from_window(&window.winit())?)
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
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .block_on()
            .map_err(|e| {
                log::error!("Failed to create device: {}", e);
                GraphicsError::DeviceError
            })?;

        let surface_capabilities = surface.get_capabilities(&adapter);

        fn is_srgb(format: wgpu::TextureFormat) -> bool {
            format == wgpu::TextureFormat::Bgra8UnormSrgb
                || format == wgpu::TextureFormat::Rgba8UnormSrgb
        }

        // Get preferred format
        let output_color_format = if surface_capabilities.formats.is_empty() {
            log::warn!("No preferred format found, using Bgra8UnormSrgb");
            wgpu::TextureFormat::Bgra8UnormSrgb
        } else {
            log::info!(
                "Preferred surface format: {:?}",
                surface_capabilities.formats[0]
            );

            // If preferred format is not Srgb, look if there are any supported Srgb formats to fall back to
            if !is_srgb(surface_capabilities.formats[0]) {
                let srgb_format = surface_capabilities
                    .formats
                    .iter()
                    .find(|f| is_srgb(**f))
                    .copied();

                if let Some(srgb_format) = srgb_format {
                    log::warn!(
                        "Preferred format is not Srgb, falling back to {:?}",
                        srgb_format
                    );
                    srgb_format
                } else {
                    log::warn!("No Srgb format found, using preferred: {:?}. This may cause issues with color accuracy.", surface_capabilities.formats[0]);
                    surface_capabilities.formats[0]
                }
            } else {
                surface_capabilities.formats[0]
            }
        };

        let output_depth_format = Some(wgpu::TextureFormat::Depth32Float);

        let mut res = Self {
            instance,
            adapter,
            device,
            surface,
            queue,
            output_color_format,
            output_depth_format,
            current_present_mode: PresentMode::AutoNoVsync,
            cache: GraphicsCache::new(),
        };

        res.configure_surface(window.winit().inner_size().into(), PresentMode::AutoNoVsync);

        Ok(res)
    }

    pub fn available_present_modes(&self) -> Vec<PresentMode> {
        self.surface
            .get_capabilities(&self.adapter)
            .present_modes
            .iter()
            .copied()
            .collect()
    }

    /// No validation of whether the present mode is available is done here.
    pub fn configure_surface(&mut self, size: (u32, u32), present_mode: PresentMode) {
        self.surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.output_color_format,
                width: size.0,
                height: size.1,
                present_mode,
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![self.output_color_format],
            },
        );

        self.current_present_mode = present_mode;
    }

    pub fn begin_frame(&self) -> Result<Frame, Box<dyn std::error::Error>> {
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
        })
    }

    pub fn end_frame<F>(&self, frame: Frame, prepresent_callback: Option<F>)
    where
        F: FnMut(),
    {
        self.queue.submit(std::iter::once(frame.encoder.finish()));

        if let Some(mut callback) = prepresent_callback {
            callback();
        }

        frame.swap_texture.present();
    }
}

impl Drop for Graphics {
    fn drop(&mut self) {
        self.device.poll(wgpu::Maintain::Wait);
    }
}
