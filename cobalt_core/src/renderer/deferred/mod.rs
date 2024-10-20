pub mod depth_buffer;
pub mod g_buffers;
pub mod material;
pub mod passes;
pub mod screen_quad;

use exports::GeometryPassDebugMode;
use ultraviolet::Mat4;

use crate::exports::{components::Transform, ecs::World};
use cobalt_graphics::context::Graphics;

use self::{
    depth_buffer::DepthBuffer,
    material::Material,
    passes::{
        color::{ColorPass, ColorPassInput},
        geometry::GeometryPass,
        geometry_debug::GeometryDebugPass,
    },
};

use super::{
    camera::Camera,
    proj_view::ProjView,
    render_pass::RenderPass,
    renderer::{FramePrepError, Renderer, RendererError},
    FrameData,
};

pub mod exports {
    pub use super::material::Material;
    pub use super::passes::geometry_debug::GeometryPassDebugMode;
    pub use super::DeferredRenderer as Renderer;
}

pub struct DeferredRenderer {
    geometry_pass: GeometryPass,
    geometry_debug_pass: GeometryDebugPass,
    color_pass: ColorPass,
    depth_buffer: DepthBuffer,
    current_output_size: (u32, u32),
}

impl DeferredRenderer {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

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
    ///
    /// Returns: (ProjView, Camera Position)
    fn get_camera(
        &self,
        world: &mut World,
        surface_dimensions: (u32, u32),
    ) -> Result<(ProjView, ultraviolet::Vec3), FramePrepError> {
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

            let proj_matrix = camera.projection_matrix(surface_dimensions);

            Ok((
                ProjView::new(view_matrix, proj_matrix),
                transform.position(),
            ))
        } else {
            Err(FramePrepError::NoCamera)
        }
    }
}

impl Renderer for DeferredRenderer {
    fn new(graphics: &Graphics, output_size: (u32, u32)) -> Result<Self, RendererError>
    where
        Self: Sized,
    {
        log::info!("Creating deferred renderer.");
        Ok(Self {
            geometry_pass: GeometryPass::new(graphics, output_size),
            geometry_debug_pass: GeometryDebugPass::new(graphics),
            color_pass: ColorPass::new(graphics, output_size),
            depth_buffer: DepthBuffer::new(graphics, output_size, Self::DEPTH_FORMAT)?,
            current_output_size: output_size,
        })
    }

    fn prep_frame<'a>(
        &mut self,
        _frame: &mut cobalt_graphics::frame::Frame,
        world: &'a mut World,
        surface_dimensions: (u32, u32),
    ) -> Result<FrameData<'a, Material>, FramePrepError> {
        let (proj_view, cam_pos) = self.get_camera(world, surface_dimensions)?;

        let frame_data = FrameData::generate(
            world,
            Some(self.depth_buffer.texture.create_view(&Default::default())),
            proj_view,
            cam_pos,
        )?;

        Ok(frame_data)
    }

    fn render(
        &mut self,
        graphics: &Graphics,
        frame: &mut cobalt_graphics::frame::Frame,
        mut frame_data: FrameData<Material>,
    ) -> Result<(), RendererError> {
        self.geometry_pass
            .draw(frame, graphics, &mut frame_data, ())?;

        #[cfg(feature = "debug_stats")]
        {
            crate::stats::Stats::global().set(
                "Geometry Pass Debug Mode",
                format!("{:?}", self.geometry_debug_pass.mode).into(),
                false,
            );
        }
        // If any debug mode is active, render it into the swap chain
        if let Some(_) = self.geometry_debug_pass.mode {
            self.geometry_debug_pass.draw(
                frame,
                graphics,
                &mut frame_data,
                (&self.geometry_pass.g_buffers, &self.depth_buffer),
            )?;
        } else {
            let camera_position = frame_data.camera_position.clone();

            self.color_pass.draw(
                frame,
                graphics,
                &mut frame_data,
                ColorPassInput {
                    geometry_buffers: &self.geometry_pass.g_buffers,
                    depth_buffer: &self.depth_buffer,
                    cam_position: camera_position,
                },
            )?;
        }

        Ok(())
    }

    fn resize_callback(
        &mut self,
        graphics: &Graphics,
        size: (u32, u32),
    ) -> Result<(), RendererError> {
        self.depth_buffer = DepthBuffer::new(graphics, size, Self::DEPTH_FORMAT)?;

        self.geometry_pass.resize_callback(graphics, size)?;
        self.color_pass.resize_callback(graphics, size)?;
        self.geometry_debug_pass.resize_callback(graphics, size)?;

        self.current_output_size = size;

        Ok(())
    }

    fn get_current_output_size(&self) -> (u32, u32) {
        self.current_output_size
    }
}
