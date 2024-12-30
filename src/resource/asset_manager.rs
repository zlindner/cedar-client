use std::{collections::HashMap, path::Path};

use nx_pkg4::{Node, NxFile};

use crate::graphics::Texture;

pub struct AssetManager {
    pub nx: HashMap<String, NxFile>,
}

impl AssetManager {
    pub fn new() -> Self {
        let mut nx = HashMap::new();

        nx.insert(
            "Map001.nx".to_string(),
            NxFile::open(Path::new("nx/Map001.nx")).unwrap(),
        );
        nx.insert(
            "UI.nx".to_string(),
            NxFile::open(Path::new("nx/UI.nx")).unwrap(),
        );

        Self { nx }
    }

    pub fn get_texture(&self, path: &str) -> Option<Texture> {
        log::info!("Getting texture for {}", path);
        let (file_name, path) = path.split_at(path.find("/").unwrap());

        let file = match self.nx.get(file_name) {
            Some(file) => file,
            None => {
                log::warn!("{} isn't open", file_name);
                return None;
            }
        };

        let root = file.root();

        // Remove the leading slash from path.
        let node = match root.get(&path[1..path.len()]) {
            Some(node) => node,
            None => {
                log::error!("Texture not found {}", path);
                return None;
            }
        };

        match Texture::load(path, node) {
            Ok(texture) => texture,
            Err(e) => {
                log::error!("Error getting texture {}: {}", path, e);
                return None;
            }
        }
    }
}
