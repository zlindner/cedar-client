use uuid::Uuid;

use crate::{component::Transform, resource::ImageHandle};

use super::{RenderCommand, RenderCommandSource, Texture};

// TODO: not a fan of this being in graphics, more like a game component.
pub struct Sprite {
    id: Uuid,
    image: ImageHandle,
    transform: Transform,
}

impl Sprite {
    pub fn new(image: ImageHandle) -> Self {
        Self {
            id: Uuid::new_v4(),
            image,
            transform: Transform::default(),
        }
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }
}

impl RenderCommandSource for Sprite {
    fn render_command(&self) -> RenderCommand {
        RenderCommand {
            id: self.id,
            texture: Texture::from_image(self.image.image()),
            transform: self.transform,
            layer: self.transform.z as usize,
        }
    }
}
