use std::{collections::HashMap, path::Path, sync::LazyLock};

use nx_pkg4::{Node, NxFile};

use crate::{component::Colour, graphics::Texture};

use super::{Font, FontDescriptor};

static NX_FILES: LazyLock<HashMap<String, NxFile>> = LazyLock::new(|| {
    let mut nx_files = HashMap::new();
    let paths = std::fs::read_dir("assets/nx").expect("nx folder should exist");

    for path in paths {
        let file_name = path.unwrap().file_name().into_string().unwrap();
        let nx_path = format!("assets/nx/{}", file_name);
        nx_files.insert(file_name, NxFile::open(Path::new(&nx_path)).unwrap());
    }

    nx_files
});

static FONTS: LazyLock<HashMap<FontDescriptor, Font>> = LazyLock::new(|| {
    let mut fonts = HashMap::new();

    // TODO fonts should be keyed by a FontKey, containing font name, size, colour.
    let descriptor = FontDescriptor::new("Arial", 13, Colour::rgb(255, 255, 255));
    fonts.insert(descriptor.clone(), Font::load(descriptor));
    fonts
});

pub struct AssetManager;

impl AssetManager {
    pub fn get_texture(path: &str) -> Option<Texture> {
        log::info!("Getting texture for {}", path);
        let (file_name, path) = path.split_at(path.find("/").unwrap());

        let file = match NX_FILES.get(file_name) {
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

    pub fn get_texture_rgba(path: &str) -> Option<Texture> {
        let mut texture = match Self::get_texture(path) {
            Some(texture) => texture,
            None => return None,
        };

        for pixel in texture.data.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }

        Some(texture)
    }

    pub fn get_font(descriptor: &FontDescriptor) -> Option<&'static Font> {
        FONTS.get(descriptor)
    }
}
