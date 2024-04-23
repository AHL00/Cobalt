pub mod g_buffers;
pub mod passes;

use ultraviolet::Mat4;

use crate::{
    exports::{components::Transform, ecs::World},
    graphics::context::Graphics,
};

use self::passes::geometry::GeometryPass;

use super::{
    camera::Camera, proj_view::ProjView, render_pass::RenderPass, renderer::Renderer, FrameData,
};

pub mod exports {
    pub use super::DeferredRenderer;
    pub use super::DeferredRendererDebugMode;
}

#[derive(Debug, Clone, Copy)]
pub enum DeferredRendererDebugMode {
    None,
    Position,
    Normal,
    AlbedoSpecular,
    Depth,
}

pub struct DeferredRenderer {
    geometry_pass: GeometryPass,
    depth_buffer: wgpu::Texture,
    current_output_size: (u32, u32),
    debug_mode: DeferredRendererDebugMode,
}

impl DeferredRenderer {
    pub fn new(output_size: (u32, u32)) -> Self {
        Self {
            geometry_pass: GeometryPass::new(output_size),
            depth_buffer: Self::generate_depth_texture(output_size),
            current_output_size: output_size,
            debug_mode: DeferredRendererDebugMode::None,
        }
    }
    
    pub fn set_debug_mode(&mut self, debug_mode: DeferredRendererDebugMode) {
        self.debug_mode = debug_mode;

        log::warn!("Debug mode for deferred renderer not yet implemented!");
    }

    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    fn generate_depth_texture(size: (u32, u32)) -> wgpu::Texture {
        Graphics::global_read()
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth Texture"),
                size: wgpu::Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                },
                view_formats: &[Self::DEPTH_FORMAT],
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: Self::DEPTH_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
            })
    }

    // TODO: Integrate into Scene system when it is implemented.
    /// Gets the camera in the scene, if there is one.
    /// If there is more than one camera, it will log a warning and return None.
    /// If the camera does not have a transform, it will log a warning and return None.
    /// If there is no enabled camera, it will log and return None.
    ///
    /// Then, it extracts the `ProjView` from the camera and returns it.
    ///
    /// If problems are encountered, it will return an error.
    fn get_camera(
        &self,
        world: &mut World,
    ) -> Result<Option<ProjView>, Box<dyn std::error::Error>> {
        let cam_query = world.query::<Camera>().unwrap();
        let mut enabled_camera_count = 0;
        let mut camera_entity = None;

        for (ent, cam) in cam_query {
            if cam.enabled {
                enabled_camera_count += 1;
            }

            // Make sure there is only one camera.
            if enabled_camera_count > 1 {
                log_once::warn_once!("More than one enabled camera entity found.");
                break;
            }

            // Make sure it has a transform.
            if let Some(_) = world.get_component::<Transform>(ent) {
                if cam.enabled {
                    camera_entity = Some(ent);
                }
                break;
            }

            log_once::warn_once!("Camera [{:?}] does not have a transform component.", ent);
        }

        if enabled_camera_count == 0 {
            log_once::warn_once!("No enabled camera entity found.");
        }

        if let Some(camera_entity) = camera_entity {
            let (transform, camera) = world
                .query_entity_mut::<(Transform, Camera)>(camera_entity)
                .expect("Camera entity not found.");

            let view_matrix = Mat4::look_at(
                transform.position(),
                transform.position() + transform.forward(),
                transform.up(),
            );

            let proj_matrix = camera.projection_matrix();

            Ok(Some(ProjView::new(view_matrix, proj_matrix)))
        } else {
            Ok(None)
        }
    }
}

impl Renderer for DeferredRenderer {
    fn render(
        &mut self,
        frame: &mut crate::graphics::frame::Frame,
        world: &mut crate::exports::ecs::World,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let proj_view = self.get_camera(world)?;

        if proj_view.is_none() {
            // No camera found, so don't render anything.
            // Errors are already logged in `get_camera`.
            return Ok(());
        }

        let proj_view = proj_view.unwrap();

        let mut frame_data = FrameData::generate(
            world,
            Some(
                self.depth_buffer
                    .create_view(&wgpu::TextureViewDescriptor::default()),
            ),
        )?;

        self.geometry_pass
            .draw(frame, &Graphics::global_read(), proj_view, &mut frame_data)?;

        // If any debug mode is active, render it into the swap chain
        match self.debug_mode {
            DeferredRendererDebugMode::None => {}
            _ => {}
        }

        Ok(())
    }

    fn resize_callback(&mut self, size: (u32, u32)) -> Result<(), Box<dyn std::error::Error>> {
        self.geometry_pass.resize_callback(size)?;
        self.depth_buffer = Self::generate_depth_texture(size);

        self.current_output_size = size;

        Ok(())
    }

    fn get_current_output_size(&self) -> (u32, u32) {
        self.current_output_size
    }
}
