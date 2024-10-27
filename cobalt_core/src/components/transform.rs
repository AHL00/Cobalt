// TODO: Systems manager that allows systems to be registered from anywhere???
// But world is not global, so how would that work?
// Maybe make world a thread safe globa?
// This is work for later, for now just hardcode updating camera data.

use std::fmt::{Debug, Formatter};

use serde::Serialize;
use ultraviolet::{Mat3, Mat4, Rotor3, Vec3};
use wgpu::{util::DeviceExt, BindGroupDescriptor};

use cobalt_ecs::component::Component;
use cobalt_graphics::{context::Graphics, HasBindGroup, HasBindGroupLayout};

fn calculate_normal_matrix(model: &Mat4, view: &Mat4) -> Mat3 {
    ((*model * *view).inversed().transposed()).truncate()
}

fn create_tranform_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            // Model matrix
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Normal matrix, use as vec3 but sent as vec4 as wgsl doesn't want to accept a vec3
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

/// A transform component.
/// Contains position, rotation, and scale.
/// Coordinate space is right-handed, with y-up.
// TODO: Fix rotations
pub struct Transform {
    position: Vec3,
    rotation: Rotor3,
    scale: Vec3,
    model_matrix: Mat4,
    normal_matrix: Mat3,
    bind_group: Option<wgpu::BindGroup>,
    model_mat_buffer: Option<wgpu::Buffer>,
    /// A normal matrix is the truncated inverse transpose of the model matrix.
    /// Used for transforming normals in the vertex shader.
    normal_mat_buffer: Option<wgpu::Buffer>,
    /// Whether the model matrix is dirty and needs to be recalculated.
    pub(crate) model_dirty: bool,
    /// This is only processed when actually rendering at the final stage, after all
    /// culling and updating has been done. Only called when it is actually drawn.
    pub(crate) buffers_dirty: bool,
}

impl Clone for Transform {
    fn clone(&self) -> Self {
        Self {
            position: self.position,
            rotation: self.rotation,
            scale: self.scale,
            model_matrix: self.model_matrix,
            normal_matrix: self.normal_matrix,
            bind_group: None,
            model_mat_buffer: None,
            normal_mat_buffer: None,
            model_dirty: self.model_dirty,
            buffers_dirty: true,
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

#[derive(serde::Serialize, serde::Deserialize)]
struct TransformSerdeBuffer {
    position: Vec3,
    rotation: Rotor3,
    scale: Vec3,
}

impl Serialize for Transform {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        let buffer = TransformSerdeBuffer {
            position: self.position,
            rotation: self.rotation,
            scale: self.scale,
        };

        serde::Serialize::serialize(&buffer, serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Transform {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        let buffer: TransformSerdeBuffer = serde::Deserialize::deserialize(deserializer)?;

        Ok(Self {
            position: buffer.position,
            rotation: buffer.rotation,
            scale: buffer.scale,
            model_matrix: Mat4::identity(),
            normal_matrix: Mat3::identity(),
            bind_group: None,
            model_mat_buffer: None,
            normal_mat_buffer: None,
            model_dirty: true,
            buffers_dirty: true,
        })
    }
}

impl Component for Transform {
    type DeContext<'a> = ();
    type SerContext<'a> = ();

    fn deserialise<'de, D>(_context: Self::DeContext<'de>, deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        serde::Deserialize::deserialize(deserializer)
    }

    fn serialize<'se, S>(&self, _context: Self::SerContext<'se>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        serde::Serialize::serialize(self, serializer)
    }
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: Vec3::zero(),
            rotation: Rotor3::identity(),
            scale: Vec3::one(),
            model_matrix: Mat4::identity(),
            normal_matrix: Mat3::identity(),
            bind_group: None,
            model_mat_buffer: None,
            normal_mat_buffer: None,
            model_dirty: true,
            buffers_dirty: true,
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
    // pub fn position_mut(&mut self) -> &mut Vec3 {
    //     self.set_dirty();
    //     &mut self.position
    // }

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

    pub fn roll(&mut self, angle: f32) {
        self.rotation =
            // Rotor3::from_rotation_between(Vec3::unit_x(), self.rotation * Vec3::unit_x())
                Rotor3::from_euler_angles(angle, 0.0, 0.0)
                * self.rotation;
        self.set_dirty();
    }

    pub fn pitch(&mut self, angle: f32) {
        self.rotation =
            // Rotor3::from_rotation_between(Vec3::unit_y(), self.rotation * Vec3::unit_y())
                Rotor3::from_euler_angles(0.0, angle, 0.0)
                * self.rotation;
        self.set_dirty();
    }

    pub fn yaw(&mut self, angle: f32) {
        self.rotation =
            // Rotor3::from_rotation_between(Vec3::unit_z(), self.rotation * Vec3::unit_z())
                Rotor3::from_euler_angles(0.0, 0.0, angle)
                * self.rotation;
        self.set_dirty();
    }

    pub fn translate(&mut self, translation: Vec3) {
        self.position += translation;
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

    fn recalc_model(&mut self) {
        let rot_mat = self.rotation.into_matrix().into_homogeneous();
        let scale_mat = Mat4::from_nonuniform_scale(self.scale);
        let translation_mat = Mat4::from_translation(self.position);

        self.model_matrix = translation_mat * scale_mat * rot_mat;

        self.model_dirty = false;
    }

    /// Gets the model matrix.
    /// If the transform is dirty, it will be recalculated on the fly.
    pub(crate) fn model_matrix(&mut self) -> &Mat4 {
        if self.model_dirty {
            self.recalc_model();
        }

        &self.model_matrix
    }

    /// Calculates and stores the normal matrix. Next time the bind group is updated, it will be sent to the GPU as a uniform.
    pub(crate) fn calculate_normal_matrix(&mut self, view: &Mat4) {
        self.normal_matrix = calculate_normal_matrix(self.model_matrix(), view);
    }

    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::unit_z()
    }

    pub fn right(&self) -> Vec3 {
        self.rotation * -Vec3::unit_x()
    }

    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::unit_y()
    }

    fn set_dirty(&mut self) {
        self.model_dirty = true;
        self.buffers_dirty = true;
    }
}

impl HasBindGroupLayout<()> for Transform {
    fn bind_group_layout<'a>(graphics: &'a Graphics, _extra: ()) -> &'a wgpu::BindGroupLayout {
        graphics
            .cache
            .bind_group_layout_cache
            .transform
            .get_or_init(|| create_tranform_bind_group_layout(&graphics.device))
    }
}

impl HasBindGroup for Transform {
    fn bind_group(&mut self, graphics: &Graphics) -> &wgpu::BindGroup {
        if self.model_dirty {
            self.recalc_model();
        }

        if self.buffers_dirty {
            #[cfg(feature = "debug_stats")]
            let start = std::time::Instant::now();

            // Lazy initialize buffers
            if let None = self.model_mat_buffer {
                self.model_mat_buffer = Some(graphics.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(Mat4::identity().as_byte_slice()),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    },
                ));
            }

            if let None = self.normal_mat_buffer {
                self.normal_mat_buffer = Some(graphics.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(
                            Mat3::identity().into_homogeneous().as_byte_slice(),
                        ),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    },
                ));
            }

            graphics.queue.write_buffer(
                self.model_mat_buffer.as_ref().unwrap(),
                0,
                bytemuck::cast_slice(self.model_matrix.as_byte_slice()),
            );

            graphics.queue.write_buffer(
                self.normal_mat_buffer.as_ref().unwrap(),
                0,
                bytemuck::cast_slice(self.normal_matrix.into_homogeneous().as_byte_slice()),
            );

            self.buffers_dirty = false;

            // Lazy initialize bind group
            if let None = self.bind_group {
                self.bind_group = Some(
                    graphics.device.create_bind_group(&BindGroupDescriptor {
                        label: None,
                        layout: Transform::bind_group_layout(graphics, ()),
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::Buffer(
                                    self.model_mat_buffer
                                        .as_ref()
                                        .unwrap()
                                        .as_entire_buffer_binding(),
                                ),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Buffer(
                                    self.normal_mat_buffer
                                        .as_ref()
                                        .unwrap()
                                        .as_entire_buffer_binding(),
                                ),
                            },
                        ],
                    }),
                );
            }

            #[cfg(feature = "debug_stats")]
            {
                use crate::stats::{Stat, Stats};

                let end = std::time::Instant::now();
                let mut stats = Stats::global();

                let (record, _) = stats.get_mut_else_default(
                    "Transform bind group write",
                    (Stat::Duration(std::time::Duration::new(0, 0)), true),
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

        self.bind_group.as_ref().unwrap()
    }
}
