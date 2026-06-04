use uuid::Uuid;

use crate::{
    component::Transform,
    graphics::{RenderCommand, RenderCommandSource},
    resource::FontDescriptor,
};

use super::TextLayout;

// TODO: placeholder text?
// TODO: font size
// TODO: colour
// TODO: alignment
pub struct TextInput {
    id: Uuid,

    pub width: u32,
    pub height: u32,

    pub text: String,
    pub changed: bool,
    pub focused: bool,
    pub masked: bool,

    pub font_descriptor: FontDescriptor,
    pub transform: Transform,

    layout: TextLayout,
}

impl TextInput {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            width,
            height,
            font_descriptor: FontDescriptor::default(),
            text: String::new(),
            changed: true,
            focused: false,
            masked: false,
            transform: Transform::default(),
            layout: TextLayout::empty(),
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

    pub fn with_masked(mut self, masked: bool) -> Self {
        self.masked = masked;
        self
    }

    pub fn with_focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.transform.x.into()
            && x <= (self.transform.x + self.width as f32).into()
            && y >= self.transform.y.into()
            && y <= (self.transform.y + self.height as f32).into()
    }

    pub fn display_text(&self) -> String {
        if self.masked {
            "*".repeat(self.text.chars().count())
        } else {
            self.text.clone()
        }
    }

    pub fn append_text(&mut self, text: &str) {
        let previous_len = self.text.len();
        self.text.push_str(text);

        if self.text.len() != previous_len {
            self.changed = true;
        }
    }

    pub fn backspace(&mut self) {
        if self.text.pop().is_some() {
            self.changed = true;
        }
    }

    pub fn set_layout(&mut self, layout: TextLayout) {
        self.layout = layout;
    }
}

impl RenderCommandSource for TextInput {
    fn append_render_commands(&self, commands: &mut Vec<RenderCommand>) {
        self.layout.append_render_commands(commands);
    }
}
