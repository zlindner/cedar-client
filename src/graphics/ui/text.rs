use uuid::Uuid;

use crate::{
    component::Transform,
    graphics::{RenderCommand, RenderCommandSource, Texture},
    resource::{Font, FontCharacter},
};

#[derive(Debug)]
pub struct Text {
    id: Uuid,
    texture: Texture,
    transform: Transform,
}

impl Text {
    pub fn new(character: &FontCharacter, font: &Font) -> Self {
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

impl RenderCommandSource for Text {
    fn render_command(&self) -> RenderCommand {
        RenderCommand {
            id: self.id,
            texture: self.texture.clone(),
            transform: self.transform,
            layer: self.transform.z as usize,
        }
    }
}
