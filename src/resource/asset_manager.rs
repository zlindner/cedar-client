use std::{collections::HashMap, path::Path, sync::Arc};

use nx_pkg4::{Node, NxFile, NxNode};

use crate::graphics::{ImageAsset, Texture};

use super::{Font, FontDescriptor};

pub struct AssetManager {
    nx_files: HashMap<String, NxFile>,
    images: HashMap<String, Arc<ImageAsset>>,
    fonts: HashMap<FontDescriptor, Font>,
}

#[derive(Clone)]
pub struct ImageHandle {
    image: Arc<ImageAsset>,
}

impl ImageHandle {
    pub fn image(&self) -> Arc<ImageAsset> {
        self.image.clone()
    }
}

impl AssetManager {
    pub fn new() -> Self {
        let mut nx_files = HashMap::new();
        let paths = std::fs::read_dir("assets/nx").expect("nx folder should exist");

        for path in paths {
            let file_name = path.unwrap().file_name().into_string().unwrap();
            let nx_path = format!("assets/nx/{}", file_name);
            nx_files.insert(file_name, NxFile::open(Path::new(&nx_path)).unwrap());
        }

        Self {
            nx_files,
            images: HashMap::new(),
            fonts: HashMap::new(),
        }
    }

    pub fn load_image(&mut self, path: &str) -> Option<ImageHandle> {
        if let Some(image) = self.images.get(path) {
            return Some(ImageHandle {
                image: image.clone(),
            });
        }

        let image = Arc::new(self.load_image_asset(path)?);
        self.images.insert(path.to_string(), image.clone());

        Some(ImageHandle { image })
    }

    pub fn create_image(
        &mut self,
        path: String,
        width: u32,
        height: u32,
        data: Vec<u8>,
        origin: Option<(i32, i32)>,
    ) -> ImageHandle {
        let image = Arc::new(ImageAsset {
            path: path.clone(),
            width,
            height,
            data,
            origin,
            layer: None,
        });
        self.images.insert(path, image.clone());

        ImageHandle { image }
    }

    pub fn get_texture(&mut self, path: &str) -> Option<Texture> {
        let handle = self.load_image(path)?;
        Some(Texture::from_image(handle.image()))
    }

    pub fn with_node<R>(&self, path: &str, f: impl FnOnce(NxNode<'_>) -> R) -> Option<R> {
        let (file_name, path) = path.split_at(path.find("/")?);
        let file = self.nx_files.get(file_name)?;
        let root = file.root();
        let node = root.get(&path[1..path.len()])?;
        Some(f(node))
    }

    fn load_image_asset(&self, path: &str) -> Option<ImageAsset> {
        log::info!("Getting image for {}", path);
        let (file_name, path) = path.split_at(path.find("/").unwrap());

        let file = match self.nx_files.get(file_name) {
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

        let bitmap_node = resolve_bitmap_node(&root, node).unwrap_or(node);

        match ImageAsset::load_with_bitmap(path, node, bitmap_node) {
            Ok(image) => image,
            Err(e) => {
                log::error!("Error getting image {}: {}", path, e);
                return None;
            }
        }
    }

    pub fn get_texture_rgba(&mut self, path: &str) -> Option<Texture> {
        let texture = match self.get_texture(path) {
            Some(texture) => texture,
            None => return None,
        };

        let mut image = (*texture.image).clone();

        for pixel in image.data.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }

        Some(Texture::from_image_asset(image))
    }

    pub fn get_font(&mut self, descriptor: &FontDescriptor) -> Option<&Font> {
        if !self.fonts.contains_key(descriptor) {
            self.fonts
                .insert(descriptor.clone(), Font::load(descriptor.clone()));
        }

        self.fonts.get(descriptor)
    }
}

fn resolve_bitmap_node<'a>(root: &'a NxNode<'a>, node: NxNode<'a>) -> Option<NxNode<'a>> {
    let source = node_string(node, "source");
    let outlink = node_string(node, "_outlink");

    resolve_link(root, source.as_deref())
        .or_else(|| resolve_link(root, outlink.as_deref()))
        .or_else(|| resolve_inlink(root, node_string(node, "_inlink").as_deref()))
}

fn resolve_link<'a>(root: &'a NxNode<'a>, link: Option<&str>) -> Option<NxNode<'a>> {
    let link = link?;
    if link.is_empty() {
        return None;
    }

    let path = link.split_once('/').map(|(_, path)| path).unwrap_or(link);
    root.get(path)
}

fn resolve_inlink<'a>(root: &'a NxNode<'a>, link: Option<&str>) -> Option<NxNode<'a>> {
    let node = resolve_link(root, link)?;

    if node.bitmap().ok().flatten().is_some() {
        Some(node)
    } else {
        None
    }
}

fn node_string(node: NxNode<'_>, child: &str) -> Option<String> {
    node.get(child).string().ok().flatten().map(str::to_string)
}
