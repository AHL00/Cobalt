use std::{
    io::{BufReader, Cursor},
    path::Path,
};

use wgpu::util::DeviceExt;

use crate::{
    assets::{Asset, AssetHandle, AssetLoadError},
    ecs::component::Component,
    engine::graphics,
    graphics::{
        vertex::UvNormalVertex,
        CreateBindGroup, HasBindGroup, HasBindGroupLayout, HasVertexBufferLayout,
    },
    transform::Transform,
};

use super::{material::{Material, MaterialTrait}, RendererPipeline, ViewProj};


pub struct MeshAsset {
    /// Buffer of NormalUvVertex
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) num_indices: u32,
    pub(crate) has_uv: bool,
}

impl Asset for MeshAsset {
    fn load(
        mut reader: BufReader<std::fs::File>,
        name: &imstr::ImString,
        path: &Path,
    ) -> Result<Self, AssetLoadError> {
        let obj_parent_dir = path.parent();

        if obj_parent_dir.is_none() {
            return Err(AssetLoadError::LoadError(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "Path [{:?}] does not have a parent directory",
                    path.file_name().unwrap()
                ),
            ))));
        }

        let obj_parent_dir = obj_parent_dir.unwrap();

        let result = tobj::load_obj_buf(
            &mut reader,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
            |p| {
                let p = obj_parent_dir.join(p);

                let mat_string = std::fs::read_to_string(&p);

                if let Err(e) = &mat_string {
                    log::error!("Could not read material file {:?}: {}", p, e);
                    return Err(tobj::LoadError::ReadError);
                }

                let mat_string = mat_string.unwrap();

                let mat = tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_string)));

                if let Err(e) = &mat {
                    log::error!("Could not load material file {:?}: {}", p, e);
                    return Err(tobj::LoadError::ReadError);
                }

                let mat = mat.unwrap();

                Ok(mat)
            },
        );

        let (models, _mats) = match result {
            Ok((models, mats)) => {
                let mats = match mats {
                    Ok(m) => m,
                    Err(e) => return Err(AssetLoadError::LoadError(Box::new(e))),
                };

                (models, mats)
            }
            Err(e) => return Err(AssetLoadError::LoadError(Box::new(e))),
        };

        if models.len() > 1 {
            log::warn!(
                "Mesh {} contains more than one model. Only the first model will be loaded.",
                name
            );
        }

        let model = &models[0];

        // Check if normals and texcoords are present
        if model.mesh.normals.len() == 0 {
            log::warn!(
                "Mesh {} does not contain normals. Normals will be generated.",
                name
            );
            // TODO: If normals aren't present, generate them
            unimplemented!("Normal generation not implemented");
        }

        if model.mesh.texcoords.len() == 0 {
            log::warn!("Mesh {} does not contain texture coordinates. There will be issues if rendering with textures.", name);
        }

        let includes_texcoords = model.mesh.texcoords.len() > 0;

        let vertex_buffer = {
            let vertices = (0..model.mesh.positions.len() / 3)
                .map(|i| UvNormalVertex {
                    position: [
                        model.mesh.positions[i * 3],
                        model.mesh.positions[i * 3 + 1],
                        model.mesh.positions[i * 3 + 2],
                    ],
                    uv: [
                        if includes_texcoords {
                            model.mesh.texcoords[i * 2]
                        } else {
                            0.0
                        },
                        if includes_texcoords {
                            1.0 - model.mesh.texcoords[i * 2 + 1]
                        } else {
                            0.0
                        },
                    ],
                    normal: [
                        model.mesh.normals[i * 3],
                        model.mesh.normals[i * 3 + 1],
                        model.mesh.normals[i * 3 + 2],
                    ],
                })
                .collect::<Vec<_>>();

            graphics()
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Mesh Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                })
        };

        let index_buffer =
            graphics()
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Mesh Index Buffer"),
                    contents: bytemuck::cast_slice(&model.mesh.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

        Ok(Self {
            vertex_buffer,
            index_buffer,
            num_indices: model.mesh.indices.len() as u32,
            has_uv: model.mesh.texcoords.len() > 0,
        })
    }
}

pub struct Mesh {
    pub mesh_asset: AssetHandle<MeshAsset>,
    // TODO: Material
    // Contains texture, color, etc.
    pub material: Material,
}

impl Mesh {
    pub fn new(mesh: AssetHandle<MeshAsset>, material: Material) -> Self {
        Self {
            mesh_asset: mesh,
            material,
        }
    }
}

impl Component for Mesh {}

pub(crate) struct MeshPipeline {
    // TODO: Add another pipeline which renders meshes with textures
    textureless_pipeline: Option<wgpu::RenderPipeline>,
}

impl MeshPipeline {
    pub fn new(graphics: &crate::graphics::Graphics) -> Self {
        let mut res = Self {
            textureless_pipeline: None,
        };

        let shader = graphics
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Mesh Vertex Shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("shaders/textureless_mesh.wgsl").into(),
                ),
            });

        let pipeline_layout =
            graphics
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Mesh Pipeline Layout"),
                    bind_group_layouts: &[
                        &Transform::bind_group_layout(),
                        &ViewProj::bind_group_layout(),
                    ],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            graphics
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Mesh Render Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[UvNormalVertex::vertex_buffer_layout()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(graphics.output_color_format.into())],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        unclipped_depth: false,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: graphics.output_depth_format.unwrap(),
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState {
                            front: wgpu::StencilFaceState::IGNORE,
                            back: wgpu::StencilFaceState::IGNORE,
                            read_mask: 0,
                            write_mask: 0,
                        },
                        bias: wgpu::DepthBiasState {
                            constant: 0,
                            slope_scale: 0.0,
                            clamp: 0.0,
                        },
                    }),
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                });

        res.textureless_pipeline = Some(render_pipeline);

        res
    }
}

impl RendererPipeline for MeshPipeline {
    fn render(
        &mut self,
        frame: &mut crate::graphics::Frame,
        world: &mut crate::ecs::World,
        view_proj: super::ViewProj,
        render_data: &super::RenderData,
    ) {
        let swap_texture_view = &frame
            .swap_texture()
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = frame.encoder();

        let view_proj_bind_group = view_proj.create_bind_group(&graphics().device);

        let mut render_pass =
            self.create_wgpu_render_pass(&mut encoder, swap_texture_view, render_data);

        render_pass.set_bind_group(1, &view_proj_bind_group, &[]);

        let query = world.query_mut::<(Mesh, Transform)>().unwrap();

        for (_, (mesh, transform)) in query {
            render_pass.set_bind_group(0, &transform.bind_group(), &[]);

            let mesh_asset = mesh.mesh_asset.borrow();

            // This is perfectly safe as long as the mesh asset is not dropped while the render pass is being executed
            // This should never be an issue, as the mesh asset is owned by the mesh component, which is owned by the world
            // The component can't be dropped while the world is borrowed. Even if multithreaded, the user of the engine would
            // never be able to drop the mesh asset while the render pass is being executed.
            let mesh_asset_unsafe = unsafe { &*(&*mesh_asset as *const MeshAsset) };

            // Set material
            mesh.material.set_uniforms(1, &mut render_pass);

            // Set pipeline
            render_pass.set_pipeline(mesh.material.get_pipeline());

            render_pass.set_vertex_buffer(0, mesh_asset_unsafe.vertex_buffer.slice(..));

            render_pass.set_index_buffer(
                mesh_asset_unsafe.index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );

            render_pass.draw_indexed(0..mesh_asset.num_indices, 0, 0..1);
        }
    }

    fn create_wgpu_render_pass<'a>(
        &self,
        encoder: &'a mut wgpu::CommandEncoder,
        swap_texture: &'a wgpu::TextureView,
        depth_view: &'a super::RenderData,
    ) -> wgpu::RenderPass<'a> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Mesh Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: swap_texture,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view.depth_view.as_ref().unwrap(),
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        })
    }
}
