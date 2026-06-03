use std::sync::Arc;

use uuid::Uuid;

use crate::{
    component::Transform,
    graphics::{ImageAsset, RenderCommand, RenderCommandSource, Texture},
    resource::FontCharacter,
};

#[derive(Debug)]
pub struct Text {
    id: Uuid,
    texture: Texture,
    transform: Transform,
}

impl Text {
    pub fn new(character: &FontCharacter, atlas: Arc<ImageAsset>) -> Self {
        Self {
            id: Uuid::new_v4(),
            texture: Texture::font(character, atlas),
            transform: Transform::default(),
        }
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }
}

impl RenderCommandSource for Text {
    fn append_render_commands(&self, commands: &mut Vec<RenderCommand>) {
        commands.push(RenderCommand {
            id: self.id,
            texture: self.texture.clone(),
            transform: self.transform,
            layer: self.transform.z as usize,
        });
    }
}
