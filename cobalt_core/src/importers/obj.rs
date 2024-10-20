use std::io::Write;

use cobalt_assets::asset::{AssetImportError, AssetImporter, AssetVerifyError};
use cobalt_graphics::vertex::UvNormalVertex;

use crate::{asset_types::mesh::MeshAssetBuffer, renderer::mesh::Mesh, types::aabb::AABB};

pub struct ObjImporter;

impl AssetImporter<Mesh> for ObjImporter {
    fn unimported_fs_type() -> cobalt_assets::asset::AssetFileSystemType {
        cobalt_assets::asset::AssetFileSystemType::File
    }

    fn verify_source(abs_path: &std::path::Path) -> Result<(), AssetVerifyError> {
        if abs_path.extension().unwrap() != "obj" {
            return Err(AssetVerifyError::InvalidFileType);
        }

        let _res = tobj::load_obj(
            abs_path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ignore_points: true,
                ignore_lines: true,
                ..Default::default()
            },
        )
        .map_err(|e| AssetVerifyError::InvalidFile(Box::new(e)))?;

        Ok(())
    }

    fn import(
        abs_input_path: &std::path::Path,
        asset_info: &cobalt_assets::manifest::AssetInfo,
        assets_dir: &std::path::Path,
    ) -> Result<cobalt_assets::manifest::ExtraAssetInfo, AssetImportError> {
        let obj_parent_dir =
            abs_input_path
                .parent()
                .ok_or(AssetImportError::LoadError(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Could not get the .obj file's parent directory."),
                ))))?;

        let file = std::fs::File::open(abs_input_path)?;

        let mut reader = std::io::BufReader::new(file);

        let result = tobj::load_obj_buf(
            &mut reader,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ignore_points: true,
                ignore_lines: true,
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

                let mat = tobj::load_mtl_buf(&mut std::io::BufReader::new(std::io::Cursor::new(
                    mat_string,
                )));

                if let Err(e) = &mat {
                    log::error!("Could not load material file {:?}: {}", p, e);
                    return Err(tobj::LoadError::ReadError);
                }

                let mat = mat.unwrap();

                Ok(mat)
            },
        );

        let (mut models, _mats) = match result {
            Ok((models, mats)) => {
                let mats = match mats {
                    Ok(m) => m,
                    Err(e) => return Err(AssetImportError::LoadError(Box::new(e))),
                };

                (models, mats)
            }
            Err(e) => return Err(AssetImportError::LoadError(Box::new(e))),
        };

        if models.len() > 1 {
            log::warn!(
                "Mesh \"{}\" contains more than one model. Only the first model will be loaded.",
                asset_info.name
            );
        }

        let model = &mut models[0];

        // Check if normals and texcoords are present
        if model.mesh.normals.len() == 0 {
            log::warn!(
                "Mesh \"{}\" does not contain normals. Normals will be generated.",
                asset_info.name
            );
            
            let normals = generate_normals(&model.mesh.positions, &model.mesh.indices);
            
            model.mesh.normals = normals;
        }
        
        if model.mesh.texcoords.len() == 0 {
            log::warn!("Mesh \"{}\" does not contain texture coordinates. There will be issues if rendering with textures.", asset_info.name);
        }

        let includes_texcoords = model.mesh.texcoords.len() > 0;

        let mut min = (0.0, 0.0, 0.0);
        let mut max = (0.0, 0.0, 0.0);

        let vertex_buffer = (0..model.mesh.positions.len() / 3)
            .map(|i| {
                // Min and max checks
                let mut is_min = false;
                if model.mesh.positions[i * 3] < min.0 {
                    min.0 = model.mesh.positions[i * 3];
                    is_min = true;
                }

                if model.mesh.positions[i * 3 + 1] < min.1 {
                    min.1 = model.mesh.positions[i * 3 + 1];
                    is_min = true;
                }

                if model.mesh.positions[i * 3 + 2] < min.2 {
                    min.2 = model.mesh.positions[i * 3 + 2];
                    is_min = true;
                }

                if !is_min {
                    if model.mesh.positions[i * 3] > max.0 {
                        max.0 = model.mesh.positions[i * 3];
                    }

                    if model.mesh.positions[i * 3 + 1] > max.1 {
                        max.1 = model.mesh.positions[i * 3 + 1];
                    }

                    if model.mesh.positions[i * 3 + 2] > max.2 {
                        max.2 = model.mesh.positions[i * 3 + 2];
                    }
                }

                UvNormalVertex {
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
                }
            })
            .collect::<Vec<_>>();

        let index_buffer = &model.mesh.indices;

        let mesh_buffer = MeshAssetBuffer {
            vertex_buffer,
            index_buffer: index_buffer.clone(),
            num_indices: model.mesh.indices.len() as u32,
            local_aabb: AABB::from_min_max(min.into(), max.into()),
            has_uv: model.mesh.texcoords.len() > 0,
        };

        let ser_bytes = bincode::serialize(&mesh_buffer)?;

        // Write the mesh buffer to the target file
        let target_path = assets_dir.join(&asset_info.relative_path);

        let mut file =
            std::fs::File::create(&target_path).map_err(|e| AssetImportError::WriteError(e))?;

        file.write_all(&ser_bytes)
            .map_err(|e| AssetImportError::WriteError(e))?;

        Ok(cobalt_assets::manifest::ExtraAssetInfo::new())
    }
}

fn generate_normals(positions: &[f32], indices: &[u32]) -> Vec<f32> {
    let mut normals = vec![0.0; positions.len()];

    for i in (0..indices.len()).step_by(3) {
        let i0 = indices[i] as usize;
        let i1 = indices[i + 1] as usize;
        let i2 = indices[i + 2] as usize;

        let v0 = [
            positions[i0 * 3],
            positions[i0 * 3 + 1],
            positions[i0 * 3 + 2],
        ];
        let v1 = [
            positions[i1 * 3],
            positions[i1 * 3 + 1],
            positions[i1 * 3 + 2],
        ];
        let v2 = [
            positions[i2 * 3],
            positions[i2 * 3 + 1],
            positions[i2 * 3 + 2],
        ];

        let e1 = [
            v1[0] - v0[0],
            v1[1] - v0[1],
            v1[2] - v0[2],
        ];
        let e2 = [
            v2[0] - v0[0],
            v2[1] - v0[1],
            v2[2] - v0[2],
        ];

        let normal = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];

        for j in 0..3 {
            normals[i0 * 3 + j] += normal[j];
            normals[i1 * 3 + j] += normal[j];
            normals[i2 * 3 + j] += normal[j];
        }
    }

    for i in (0..normals.len()).step_by(3) {
        let len = (normals[i] * normals[i] + normals[i + 1] * normals[i + 1] + normals[i + 2] * normals[i + 2]).sqrt();

        normals[i] /= len;
        normals[i + 1] /= len;
        normals[i + 2] /= len;
    }

    normals
}