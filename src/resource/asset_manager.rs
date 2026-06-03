use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, LazyLock, Mutex},
};

use nx_pkg4::{Node, NxFile};

use crate::{
    component::Colour,
    graphics::{ImageAsset, Texture},
};

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

#[derive(Clone)]
pub struct ImageHandle {
    image: Arc<ImageAsset>,
}

impl ImageHandle {
    pub fn image(&self) -> Arc<ImageAsset> {
        self.image.clone()
    }
}

static IMAGES: LazyLock<Mutex<HashMap<String, Arc<ImageAsset>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

impl AssetManager {
    pub fn load_image(path: &str) -> Option<ImageHandle> {
        let mut images = IMAGES.lock().expect("image cache should lock");

        if let Some(image) = images.get(path) {
            return Some(ImageHandle {
                image: image.clone(),
            });
        }

        let image = Arc::new(Self::load_image_asset(path)?);
        images.insert(path.to_string(), image.clone());

        Some(ImageHandle { image })
    }

    pub fn get_texture(path: &str) -> Option<Texture> {
        let handle = Self::load_image(path)?;
        Some(Texture::from_image(handle.image()))
    }

    fn load_image_asset(path: &str) -> Option<ImageAsset> {
        log::info!("Getting image for {}", path);
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
                log::error!("Image not found {}", path);
                return None;
            }
        };

        match ImageAsset::load(path, node) {
            Ok(image) => image,
            Err(e) => {
                log::error!("Error getting image {}: {}", path, e);
                return None;
            }
        }
    }

    pub fn get_texture_rgba(path: &str) -> Option<Texture> {
        let texture = match Self::get_texture(path) {
            Some(texture) => texture,
            None => return None,
        };

        let mut image = (*texture.image).clone();

        for pixel in image.data.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }

        Some(Texture::from_image_asset(image))
    }

    pub fn get_font(descriptor: &FontDescriptor) -> Option<&'static Font> {
        FONTS.get(descriptor)
    }
}
