use nx_pkg4::{Node, NxError, NxNode};

use crate::resource::Font;

#[derive(Clone)]
pub struct ImageAsset {
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub origin: Option<(i32, i32)>,
    pub layer: Option<i64>,
}

impl ImageAsset {
    pub fn load(path: &str, node: NxNode) -> Result<Option<Self>, NxError> {
        let origin = match node.get("origin") {
            Some(child) => child.vector()?,
            None => None,
        };

        let layer = match node.get("z") {
            Some(child) => child.integer()?,
            None => None,
        };

        let bitmap = match node.bitmap()? {
            Some(bitmap) => bitmap,
            None => {
                log::warn!("{} isn't a bitmap", path);
                return Ok(None);
            }
        };

        Ok(Some(Self {
            path: path.to_string(),
            width: bitmap.width.into(),
            height: bitmap.height.into(),
            data: bitmap.data,
            origin,
            layer,
        }))
    }

    pub fn font(font: &Font) -> Self {
        Self {
            path: font.texture_key.clone(),
            width: font.width,
            height: font.height,
            data: font.data.clone(), // TODO: fix this
            origin: None,
            layer: None,
        }
    }
}
