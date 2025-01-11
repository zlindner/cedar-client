use uuid::Uuid;

use crate::{
    component::Transform,
    graphics::{RenderableV2, Texture},
    resource::{FontCharacter, FontData},
};

#[derive(Debug)]
pub struct Text {
    id: Uuid,
    texture: Texture,
    transform: Transform,
}

impl Text {
    pub fn new(character: &FontCharacter, font: &FontData) -> Self {
        Self {
            id: Uuid::new_v4(),
            texture: Texture::font(character, font),
            transform: Transform::default(),
        }
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }
}

impl RenderableV2 for Text {
    fn id(&self) -> &uuid::Uuid {
        &self.id
    }

    fn texture(&self) -> &Texture {
        &self.texture
    }

    fn transform(&self) -> &Transform {
        &self.transform
    }
}
