use std::{
    fmt::format, io::{BufReader, Cursor}, path::Path
};

use wgpu::util::DeviceExt;

use crate::{
    assets::{Asset, AssetLoadError},
    engine::graphics,
    graphics::vertex::{NormalUvVertex, NormalVertex, UvVertex},
};

pub struct MeshAsset {
    /// Buffer of NormalUvVertex, or NormalVertex if the mesh does not have texture coordinates
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
            &tobj::GPU_LOAD_OPTIONS,
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

        let (models, mats) = match result {
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
            log::warn!("Mesh {} does not contain texture coordinates. The mesh will not have texture coordinates.", name);
        }

        let vertex_buffer = if model.mesh.texcoords.len() > 0 {
            let mut vertices = Vec::new();
            for i in 0..model.mesh.indices.len() / 3 {
                vertices.push(NormalUvVertex {
                    position: [
                        model.mesh.positions[model.mesh.indices[i * 3] as usize * 3],
                        model.mesh.positions[model.mesh.indices[i * 3] as usize * 3 + 1],
                        model.mesh.positions[model.mesh.indices[i * 3] as usize * 3 + 2],
                    ],
                    normal: [
                        model.mesh.normals[model.mesh.indices[i * 3] as usize * 3],
                        model.mesh.normals[model.mesh.indices[i * 3] as usize * 3 + 1],
                        model.mesh.normals[model.mesh.indices[i * 3] as usize * 3 + 2],
                    ],
                    uv: [
                        model.mesh.texcoords[model.mesh.indices[i * 3 + 1] as usize * 2],
                        model.mesh.texcoords[model.mesh.indices[i * 3 + 1] as usize * 2 + 1],
                    ],
                })
            }

            graphics()
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Mesh Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                })
        } else {
            let mut vertices = Vec::new();
            for i in 0..model.mesh.indices.len() / 3 {
                vertices.push(NormalVertex {
                    position: [
                        model.mesh.positions[model.mesh.indices[i * 3] as usize * 3],
                        model.mesh.positions[model.mesh.indices[i * 3] as usize * 3 + 1],
                        model.mesh.positions[model.mesh.indices[i * 3] as usize * 3 + 2],
                    ],
                    normal: [
                        model.mesh.normals[model.mesh.indices[i * 3] as usize * 3],
                        model.mesh.normals[model.mesh.indices[i * 3] as usize * 3 + 1],
                        model.mesh.normals[model.mesh.indices[i * 3] as usize * 3 + 2],
                    ],
                })
            }

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
