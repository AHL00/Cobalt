// TODO: Systems manager that allows systems to be registered from anywhere???
// But world is not global, so how would that work?
// Maybe make world a thread safe globa?
// This is work for later, for now just hardcode updating camera data.

use std::{
    fmt::{Debug, Formatter},
    sync::LazyLock,
};

use ultraviolet::{Mat4, Rotor3, Vec3};
use wgpu::util::DeviceExt;

use crate::{
    ecs::component::Component,
    graphics::{context::Graphics, HasBindGroup, HasBindGroupLayout},
    stats::{Stat, Stats},
};

static TRANSFORM_BIND_GROUP_LAYOUT: LazyLock<wgpu::BindGroupLayout> = LazyLock::new(|| {
    Graphics::global_read()
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

/// A transform component.
/// Contains position, rotation, and scale.
/// Coordinate space is right-handed, with y-up.
// TODO: Fix rotations
pub struct Transform {
    position: Vec3,
    rotation: Rotor3,
    scale: Vec3,
    model_matrix: Mat4,
    bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
    /// Whether the model matrix is dirty and needs to be recalculated.
    pub(crate) model_dirty: bool,
    /// This is only processed when actually rendering at the final stage, after all
    /// culling and updating has been done. Only called when it is actually drawn.
    pub(crate) buffer_dirty: bool,
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Transform {
    fn clone(&self) -> Self {
        let buffer =
            Graphics::global_read()
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(self.model_matrix.as_byte_slice()),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let bind_group =
            Graphics::global_read()
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &*TRANSFORM_BIND_GROUP_LAYOUT,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
                    }],
                });

        Self {
            position: self.position,
            rotation: self.rotation,
            scale: self.scale,
            model_matrix: self.model_matrix,
            bind_group,
            buffer,
            model_dirty: self.model_dirty,
            buffer_dirty: self.buffer_dirty,
        }
    }
}

impl Debug for Transform {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Transform")
            .field("position", &self.position)
            .field("rotation", &self.rotation)
            .field("scale", &self.scale)
            .field("dirty", &self.model_dirty)
            .finish()
    }
}

impl Component for Transform {}

impl Transform {
    pub fn new() -> Self {
        let buffer =
            Graphics::global_read()
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(Mat4::identity().as_byte_slice()),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let bind_group =
            Graphics::global_read()
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &*TRANSFORM_BIND_GROUP_LAYOUT,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
                    }],
                });

        Self {
            position: Vec3::zero(),
            rotation: Rotor3::identity(),
            scale: Vec3::one(),
            model_matrix: Mat4::identity(),
            bind_group,
            buffer,
            model_dirty: true,
            buffer_dirty: true,
        }
    }

    pub fn with_position(position: Vec3) -> Self {
        Self {
            position,
            ..Self::new()
        }
    }

    pub fn with_rotation(rotation: Rotor3) -> Self {
        Self {
            rotation,
            ..Self::new()
        }
    }

    pub fn with_scale(scale: Vec3) -> Self {
        Self {
            scale,
            ..Self::new()
        }
    }

    pub fn with_position_rotation_scale(position: Vec3, rotation: Rotor3, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
            ..Self::new()
        }
    }

    /// Gets the scale.
    pub fn position(&self) -> Vec3 {
        self.position
    }

    /// Gets a mutable reference to the position.
    // Marks the transform as dirty, which means the model matrix will be recalculated.
    pub fn position_mut(&mut self) -> &mut Vec3 {
        self.set_dirty();
        &mut self.position
    }

    /// Gets the rotation.
    pub fn rotation(&self) -> Rotor3 {
        self.rotation
    }

    /// Gets a mutable reference to the rotation.
    /// Marks the transform as dirty, which means the model matrix will be recalculated.
    pub fn rotation_mut(&mut self) -> &mut Rotor3 {
        self.set_dirty();
        &mut self.rotation
    }

    /// Rotates the transform around a center point.
    /// center: The point to rotate around. Coordinate is relative to transform.
    /// rotations: The rotations to apply.
    pub fn rotate(&mut self, center: Vec3, rotations: Vec3) {
        let center = self.position + center;
        let rot = Rotor3::from_rotation_between(Vec3::unit_z(), center - self.position);
        let rot = Rotor3::from_euler_angles(rotations.x, rotations.y, rotations.z) * rot;
        let rot = Rotor3::from_rotation_between(center - self.position, Vec3::unit_z()) * rot;

        self.rotation = rot * self.rotation;
        self.set_dirty();
    }

    pub fn rotate_x(&mut self, angle: f32) {
        self.rotation =
            Rotor3::from_rotation_between(Vec3::unit_x(), self.rotation * Vec3::unit_x())
                * Rotor3::from_euler_angles(angle, 0.0, 0.0)
                * self.rotation;
        self.set_dirty();
    }

    pub fn rotate_y(&mut self, angle: f32) {
        self.rotation =
            Rotor3::from_rotation_between(Vec3::unit_y(), self.rotation * Vec3::unit_y())
                * Rotor3::from_euler_angles(0.0, angle, 0.0)
                * self.rotation;
        self.set_dirty();
    }

    pub fn rotate_z(&mut self, angle: f32) {
        self.rotation =
            Rotor3::from_rotation_between(Vec3::unit_z(), self.rotation * Vec3::unit_z())
                * Rotor3::from_euler_angles(0.0, 0.0, angle)
                * self.rotation;
        self.set_dirty();
    }

    /// Gets the scale.
    pub fn scale(&self) -> Vec3 {
        self.scale
    }

    /// Gets a mutable reference to the scale.
    /// Marks the transform as dirty, which means the model matrix will be recalculated.
    pub fn scale_mut(&mut self) -> &mut Vec3 {
        self.set_dirty();
        &mut self.scale
    }

    fn recalculate_model_matrix(&mut self) {
        let rot_mat = self.rotation.into_matrix().into_homogeneous();
        let scale_mat = Mat4::from_nonuniform_scale(self.scale);
        let translation_mat = Mat4::from_translation(self.position);

        self.model_matrix = translation_mat * scale_mat * rot_mat;

        self.model_dirty = false;
    }

    // Gets the model matrix.
    // If the transform is dirty, it will be recalculated on the fly.
    pub fn model_matrix(&mut self) -> &Mat4 {
        if self.model_dirty {
            self.recalculate_model_matrix();
        }

        &self.model_matrix
    }

    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::unit_z()
    }

    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::unit_x()
    }

    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::unit_y()
    }

    fn set_dirty(&mut self) {
        self.model_dirty = true;
        self.buffer_dirty = true;
    }
}

impl HasBindGroupLayout for Transform {
    fn bind_group_layout() -> &'static wgpu::BindGroupLayout {
        &TRANSFORM_BIND_GROUP_LAYOUT
    }
}

impl HasBindGroup for Transform {
    fn bind_group(&mut self, graphics: &Graphics) -> &wgpu::BindGroup {
        if self.model_dirty {
            self.recalculate_model_matrix();
        }

        if self.buffer_dirty {
            #[cfg(feature = "debug_perf_stats")]
            let start = std::time::Instant::now();

            graphics.queue.write_buffer(
                &self.buffer,
                0,
                bytemuck::cast_slice(self.model_matrix.as_byte_slice()),
            );
            self.buffer_dirty = false;

            #[cfg(feature = "debug_perf_stats")]
            {
                let end = std::time::Instant::now();
                let mut stats = Stats::global();

                let (record, _) = stats.get_mut_else_default(
                    "Transform::bind_group::write_buffer::Duration",
                    Stat::Duration(std::time::Duration::new(0, 0)),
                );

                match record {
                    Stat::Duration(value) => {
                        // Add this duration to the frame total
                        *value += end - start;
                    }
                    _ => unreachable!(),
                };
            }
        }

        &self.bind_group
    }
}
