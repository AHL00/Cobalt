
use std::path::Path;

use crate::{assets::{Asset, AssetLoadError}, graphics::vertex::UvVertex};


pub struct Model {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

impl Asset for Model {
    fn load(data: std::io::BufReader<std::fs::File>) -> Result<Self, AssetLoadError>  {
        

        todo!()
    }
}