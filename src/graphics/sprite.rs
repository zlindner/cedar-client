use uuid::Uuid;

use crate::{component::Transform, resource::ImageHandle};

use super::{RenderableV2, Texture};

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

impl RenderableV2 for Sprite {
    fn id(&self) -> &Uuid {
        &self.id
    }

    fn texture(&self) -> Texture {
        Texture::from_image(self.image.image())
    }

    fn transform(&self) -> &Transform {
        &self.transform
    }
}
