use std::sync::Arc;

use uuid::Uuid;

use crate::{
    component::Transform,
    graphics::{ImageAsset, RenderCommand, RenderCommandSource, Texture},
    resource::{Font, FontCharacter},
};

#[derive(Debug)]
pub struct TextGlyph {
    id: Uuid,
    texture: Texture,
    transform: Transform,
}

impl TextGlyph {
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

impl RenderCommandSource for TextGlyph {
    fn append_render_commands(&self, commands: &mut Vec<RenderCommand>) {
        commands.push(RenderCommand {
            id: self.id,
            texture: self.texture.clone(),
            transform: self.transform,
            layer: self.transform.z as usize,
        });
    }
}

pub struct TextLayout {
    glyphs: Vec<TextGlyph>,
    pub width: f32,
    pub height: f32,
}

impl TextLayout {
    pub fn empty() -> Self {
        Self {
            glyphs: Vec::new(),
            width: 0.0,
            height: 0.0,
        }
    }

    pub fn new(text: &str, origin: &Transform, font: &Font) -> Self {
        let mut layout = Self::empty();
        let atlas = Arc::new(ImageAsset::font(font));
        let mut current_pos = 0.0;

        layout.height = font.line_height;

        for input_character in text.chars() {
            let Some(character) = font.characters.get(&input_character) else {
                current_pos += font.advance(input_character);
                layout.width = layout.width.max(current_pos);
                continue;
            };

            let transform = Transform::from_xyz(
                origin.x + current_pos + character.left_bearing,
                origin.y + font.line_height - character.top_bearing,
                origin.z,
            );

            // TODO: append any x/y padding from input.
            let glyph = TextGlyph::new(character, atlas.clone()).with_transform(transform);
            layout.glyphs.push(glyph);

            current_pos += character.advance;
            layout.width = layout.width.max(current_pos);
        }

        layout
    }
}

impl RenderCommandSource for TextLayout {
    fn append_render_commands(&self, commands: &mut Vec<RenderCommand>) {
        for glyph in &self.glyphs {
            glyph.append_render_commands(commands);
        }
    }
}
