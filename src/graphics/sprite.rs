use uuid::Uuid;

use crate::{component::Transform, resource::AssetManager};

use super::{RenderableV2, Texture};

// TODO: not a fan of this being in graphics, more like a game component.
pub struct Sprite {
    id: Uuid,
    texture: Texture,
    transform: Transform,
}

impl Sprite {
    pub fn new(nx_path: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            texture: AssetManager::get_texture(nx_path).unwrap(),
            transform: Transform::default(),
        }
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }
}

impl RenderableV2 for Sprite {
    fn id(&self) -> &Uuid {
        &self.id
    }

    fn texture(&self) -> &Texture {
        &self.texture
    }

    fn transform(&self) -> &Transform {
        &self.transform
    }
}
