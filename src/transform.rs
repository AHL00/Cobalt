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
    engine::graphics,
    graphics::{HasBindGroup, HasBindGroupLayout},
};

static TRANSFORM_BIND_GROUP_LAYOUT: LazyLock<wgpu::BindGroupLayout> = LazyLock::new(|| {
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

/// A transform component.
/// Contains position, rotation, and scale.
// Coordinate space is right-handed, with y-up.
pub struct Transform {
    position: Vec3,
    rotation: Rotor3,
    scale: Vec3,
    model_matrix: Mat4,
    bind_group: Option<wgpu::BindGroup>,
    model_dirty: bool,
    bind_group_dirty: bool,
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
        Self {
            position: Vec3::zero(),
            rotation: Rotor3::identity(),
            scale: Vec3::one(),
            model_matrix: Mat4::identity(),
            bind_group: None,
            model_dirty: true,
            bind_group_dirty: true,
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
        self.model_dirty = true;
        &mut self.position
    }

    /// Gets the rotation.
    pub fn rotation(&self) -> Rotor3 {
        self.rotation
    }

    /// Gets a mutable reference to the rotation.
    /// Marks the transform as dirty, which means the model matrix will be recalculated.
    pub fn rotation_mut(&mut self) -> &mut Rotor3 {
        self.model_dirty = true;
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
        self.model_dirty = true;
    }

    pub fn rotate_x(&mut self, angle: f32) {
        self.rotation = Rotor3::from_rotation_between(Vec3::unit_x(), self.rotation * Vec3::unit_x())
            * Rotor3::from_euler_angles(angle, 0.0, 0.0)
            * self.rotation;
        self.model_dirty = true;
    }

    pub fn rotate_y(&mut self, angle: f32) {
        self.rotation = Rotor3::from_rotation_between(Vec3::unit_y(), self.rotation * Vec3::unit_y())
            * Rotor3::from_euler_angles(0.0, angle, 0.0)
            * self.rotation;
        self.model_dirty = true;
    }

    pub fn rotate_z(&mut self, angle: f32) {
        self.rotation = Rotor3::from_rotation_between(Vec3::unit_z(), self.rotation * Vec3::unit_z())
            * Rotor3::from_euler_angles(0.0, 0.0, angle)
            * self.rotation;
        self.model_dirty = true;
    }

    /// Gets the scale.
    pub fn scale(&self) -> Vec3 {
        self.scale
    }

    /// Gets a mutable reference to the scale.
    /// Marks the transform as dirty, which means the model matrix will be recalculated.
    pub fn scale_mut(&mut self) -> &mut Vec3 {
        self.model_dirty = true;
        &mut self.scale
    }

    fn recalculate_model_matrix(&mut self) {
        let rot_mat = self.rotation.into_matrix().into_homogeneous();
        let scale_mat = Mat4::from_nonuniform_scale(self.scale);
        let translation_mat = Mat4::from_translation(self.position);

        self.model_matrix = translation_mat * scale_mat * rot_mat;

        self.model_dirty = false;

        self.bind_group_dirty = true;
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
}

impl HasBindGroupLayout for Transform {
    fn bind_group_layout() -> &'static wgpu::BindGroupLayout {
        &TRANSFORM_BIND_GROUP_LAYOUT
    }
}

impl HasBindGroup for Transform {
    fn bind_group(&mut self) -> &wgpu::BindGroup {
        if self.model_dirty {
            self.recalculate_model_matrix();
        }

        if self.bind_group_dirty {
            self.bind_group = Some(
                graphics()
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: None,
                        layout: &*TRANSFORM_BIND_GROUP_LAYOUT,
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(
                                graphics()
                                    .device
                                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                        label: None,
                                        contents: bytemuck::cast_slice(
                                            self.model_matrix.as_byte_slice(),
                                        ),
                                        usage: wgpu::BufferUsages::UNIFORM
                                            | wgpu::BufferUsages::COPY_DST,
                                    })
                                    .as_entire_buffer_binding(),
                            ),
                        }],
                    }),
            );

            self.bind_group_dirty = false;
        }

        self.bind_group.as_ref().unwrap()
    }
}
