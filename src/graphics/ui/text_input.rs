use uuid::Uuid;

use crate::{
    component::Transform,
    graphics::{RenderCommand, RenderCommandSource},
    resource::FontDescriptor,
};

use super::Text;

// TODO: placeholder text?
// TODO: font size
// TODO: colour
// TODO: alignment
pub struct TextInput {
    id: Uuid,

    width: u32,
    height: u32,

    pub text: String,
    pub changed: bool,

    pub font_descriptor: FontDescriptor,
    pub transform: Transform,

    glyphs: Vec<Text>,
}

impl TextInput {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            width,
            height,
            font_descriptor: FontDescriptor::default(),
            text: "TEST123".to_string(),
            changed: true,
            transform: Transform::default(),
            glyphs: Vec::new(),
        }
    }

    pub fn with_font(mut self, font_descriptor: FontDescriptor) -> Self {
        self.font_descriptor = font_descriptor;
        self
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    pub fn set_glyphs(&mut self, glyphs: Vec<Text>) {
        self.glyphs = glyphs;
    }
}

impl RenderCommandSource for TextInput {
    fn append_render_commands(&self, commands: &mut Vec<RenderCommand>) {
        for glyph in &self.glyphs {
            glyph.append_render_commands(commands);
        }
    }
}
