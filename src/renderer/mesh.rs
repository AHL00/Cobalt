use std::{
    io::{BufReader, Cursor},
    path::Path,
};

use wgpu::util::DeviceExt;

use crate::{
    assets::{AssetLoadError, AssetTrait},
    engine::graphics,
    graphics::vertex::UvNormalVertex,
};

pub struct MeshAsset {
    /// Buffer of NormalUvVertex
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) num_indices: u32,
    pub(crate) has_uv: bool,
}

impl AssetTrait for MeshAsset {
    fn load_from_file(
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
            // TODO: Fix bug that happens when loading materials
            // with spaces in their names. This is an issue with
            // the tobj crate.
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
