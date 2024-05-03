pub mod depth_buffer;
pub mod g_buffers;
pub mod passes;
pub mod screen_quad;

use std::error::Error;

use exports::GeometryPassDebugMode;
use ultraviolet::Mat4;

use crate::{
    exports::{components::Transform, ecs::World},
    graphics::context::Graphics,
    stats::Stats,
};

use self::{
    depth_buffer::DepthBuffer,
    passes::{geometry::GeometryPass, geometry_debug::GeometryDebugPass},
};

use super::{
    camera::Camera,
    proj_view::ProjView,
    render_pass::RenderPass,
    renderer::{FramePrepError, RenderError, Renderer},
    FrameData,
};

pub mod exports {
    pub use super::passes::geometry_debug::GeometryPassDebugMode;
    pub use super::DeferredRenderer;
}

pub struct DeferredRenderer {
    geometry_pass: GeometryPass,
    geometry_debug_pass: GeometryDebugPass,
    depth_buffer: DepthBuffer,
    current_output_size: (u32, u32),
}

impl DeferredRenderer {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn new(output_size: (u32, u32)) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            geometry_pass: GeometryPass::new(output_size),
            geometry_debug_pass: GeometryDebugPass::new(),
            depth_buffer: DepthBuffer::new(output_size, Self::DEPTH_FORMAT)?,
            current_output_size: output_size,
        })
    }

    // Set None to disable debug mode.
    pub fn set_debug_mode(&mut self, debug_mode: Option<GeometryPassDebugMode>) {
        self.geometry_debug_pass.mode = debug_mode;
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
    fn get_camera(&self, world: &mut World) -> Result<ProjView, FramePrepError> {
        let cam_query = world.query::<Camera>().unwrap();
        let mut enabled_camera_count = 0;
        let mut camera_entity = None;

        for (ent, cam) in cam_query {
            if cam.enabled {
                enabled_camera_count += 1;
            }

            // Make sure there is only one camera.
            if enabled_camera_count > 1 {
                return Err(FramePrepError::MultipleCameras);
            }

            camera_entity = Some(ent);
        }
        
        if let Some(camera_entity) = camera_entity {
            if let None = world.get_component::<Transform>(camera_entity) {
                return Err(FramePrepError::NoCamTransform);
            }

            let (transform, camera) = world
                .query_entity_mut::<(Transform, Camera)>(camera_entity)
                .expect("Camera entity components not found. This should never happen.");

            let view_matrix = Mat4::look_at(
                transform.position(),
                transform.position() + transform.forward(),
                transform.up(),
            );

            let proj_matrix = camera.projection_matrix();

            Ok(ProjView::new(view_matrix, proj_matrix))
        } else {
            Err(FramePrepError::NoCamera)
        }
    }
}

impl Renderer for DeferredRenderer {
    fn prep_frame<'a>(
        &mut self,
        _frame: &mut crate::graphics::frame::Frame,
        world: &'a mut World,
    ) -> Result<FrameData<'a>, FramePrepError> {
        let proj_view = self.get_camera(world)?;

        let frame_data = FrameData::generate(
            world,
            Some(self.depth_buffer.texture.create_view(&Default::default())),
            proj_view,
        )?;

        Ok(frame_data)
    }

    fn render(
        &mut self,
        frame: &mut crate::graphics::frame::Frame,
        mut frame_data: FrameData,
    ) -> Result<(), RenderError> {
        self.geometry_pass
            .draw(frame, &Graphics::global_read(), &mut frame_data, ())?;

        #[cfg(feature = "debug_stats")]
        {
            Stats::global().set(
                "Geometry Pass Debug Mode",
                format!("{:?}", self.geometry_debug_pass.mode).into(),
                false,
            );
        }
        // If any debug mode is active, render it into the swap chain
        if let Some(_) = self.geometry_debug_pass.mode {
            self.geometry_debug_pass.draw(
                frame,
                &Graphics::global_read(),
                &mut frame_data,
                (&self.geometry_pass.g_buffers, &self.depth_buffer),
            )?;
        } else {
            // Read render pass
        }

        Ok(())
    }

    fn resize_callback(&mut self, size: (u32, u32)) -> Result<(), Box<dyn Error>> {
        self.geometry_pass.resize_callback(size)?;
        self.depth_buffer = DepthBuffer::new(size, Self::DEPTH_FORMAT)?;

        self.current_output_size = size;

        Ok(())
    }

    fn get_current_output_size(&self) -> (u32, u32) {
        self.current_output_size
    }
}
