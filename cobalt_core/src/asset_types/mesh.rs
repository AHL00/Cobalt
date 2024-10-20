use std::io::{BufReader, Read};

use cobalt_assets::{
    asset::{AssetReadError, AssetTrait},
    server::AssetLoadError,
};
use cobalt_graphics::vertex::UvNormalVertex;
use wgpu::util::DeviceExt;

use crate::{renderer::mesh::Mesh, types::aabb::AABB};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MeshAssetBuffer {
    pub index_buffer: Vec<u32>,
    pub vertex_buffer: Vec<UvNormalVertex>,
    pub num_indices: u32,
    pub local_aabb: AABB,
    pub has_uv: bool,
}

impl AssetTrait for Mesh {
    fn type_name() -> String {
        "Mesh".to_string()
    }

    fn imported_fs_type() -> cobalt_assets::asset::AssetFileSystemType {
        cobalt_assets::asset::AssetFileSystemType::File
    }

    fn read(
        asset_info: &cobalt_assets::manifest::AssetInfo,
        assets_dir: &std::path::Path,
        graphics: &cobalt_graphics::context::Graphics,
    ) -> Result<Self, AssetReadError> {
        let abs_path = assets_dir.join(&asset_info.relative_path);

        let data = if let Some(_) = asset_info.pack.compression {
            let reader = BufReader::new(std::fs::File::open(&abs_path)?);

            let mut decoder = zstd::Decoder::new(reader).map_err(|e| AssetReadError::Io(e))?;

            let mut decoded = Vec::new();
            decoder
                .read_to_end(&mut decoded)
                .map_err(|e| AssetReadError::Io(e))?;

            decoded
        } else {
            std::fs::read(&abs_path)?
        };

        let mesh_buffer: MeshAssetBuffer =
            bincode::deserialize(&data).map_err(|e| AssetReadError::DeserializeError(e))?;

        let index_buffer = graphics
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&mesh_buffer.index_buffer),
                usage: wgpu::BufferUsages::INDEX,
            });

        let vertex_buffer = graphics
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&mesh_buffer.vertex_buffer),
                usage: wgpu::BufferUsages::VERTEX,
            });

        Ok(Mesh {
            index_buffer,
            vertex_buffer,
            num_indices: mesh_buffer.num_indices,
            local_aabb: mesh_buffer.local_aabb,
            has_uv: mesh_buffer.has_uv,
        })
    }
}
