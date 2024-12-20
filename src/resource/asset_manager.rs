use std::{collections::HashMap, path::Path};

use nx_pkg4::{Node, NxBitmap, NxFile};

pub struct AssetManager {
    nx: HashMap<String, NxFile>,
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

    pub fn get_bitmap(&self, path: &str) -> Option<NxBitmap> {
        let (file_name, path) = path.split_at(path.find("/").unwrap());

        let file = match self.nx.get(file_name) {
            Some(file) => file,
            None => {
                log::warn!("{} isn't open", file_name);
                return None;
            }
        };

        // Remove the leading slash.
        let path = &path[1..path.len()];

        match file.root().get(path).bitmap() {
            Ok(Some(bitmap)) => Some(bitmap),
            Ok(None) => {
                log::error!("Bitmap not found {}", path);
                None
            }
            Err(e) => {
                log::error!("Error getting bitmap {}: {}", path, e);
                None
            }
        }
    }
}
