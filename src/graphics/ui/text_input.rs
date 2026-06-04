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
    caret_index: usize,
    max_length: Option<usize>,

    pub font_descriptor: FontDescriptor,
    pub transform: Transform,

    layout: TextLayout,
    caret_layout: TextLayout,
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
            caret_index: 0,
            max_length: None,
            transform: Transform::default(),
            layout: TextLayout::empty(),
            caret_layout: TextLayout::empty(),
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

    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
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
        for character in text.chars() {
            if !self.below_limit() {
                break;
            }

            let byte_index = byte_index_at_char(&self.text, self.caret_index);
            self.text.insert(byte_index, character);
            self.caret_index += 1;
            self.changed = true;
        }
    }

    pub fn backspace(&mut self) {
        if self.caret_index == 0 {
            return;
        }

        let start = byte_index_at_char(&self.text, self.caret_index - 1);
        let end = byte_index_at_char(&self.text, self.caret_index);
        self.text.replace_range(start..end, "");
        self.caret_index -= 1;
        self.changed = true;
    }

    pub fn delete(&mut self) {
        if self.caret_index >= self.text_length() {
            return;
        }

        let start = byte_index_at_char(&self.text, self.caret_index);
        let end = byte_index_at_char(&self.text, self.caret_index + 1);
        self.text.replace_range(start..end, "");
        self.changed = true;
    }

    pub fn move_caret_left(&mut self) {
        if self.caret_index > 0 {
            self.caret_index -= 1;
            self.changed = true;
        }
    }

    pub fn move_caret_right(&mut self) {
        if self.caret_index < self.text_length() {
            self.caret_index += 1;
            self.changed = true;
        }
    }

    pub fn move_caret_to_start(&mut self) {
        if self.caret_index != 0 {
            self.caret_index = 0;
            self.changed = true;
        }
    }

    pub fn move_caret_to_end(&mut self) {
        let text_length = self.text_length();

        if self.caret_index != text_length {
            self.caret_index = text_length;
            self.changed = true;
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        if self.focused != focused {
            self.focused = focused;
            self.changed = true;
        }
    }

    pub fn set_layout(&mut self, layout: TextLayout, caret_layout: TextLayout) {
        self.layout = layout;
        self.caret_layout = caret_layout;
    }

    pub fn caret_index(&self) -> usize {
        self.caret_index
    }

    fn below_limit(&self) -> bool {
        self.max_length
            .map(|max_length| self.text_length() < max_length)
            .unwrap_or(true)
    }

    fn text_length(&self) -> usize {
        self.text.chars().count()
    }
}

impl RenderCommandSource for TextInput {
    fn append_render_commands(&self, commands: &mut Vec<RenderCommand>) {
        self.layout.append_render_commands(commands);

        if self.focused {
            self.caret_layout.append_render_commands(commands);
        }
    }
}

fn byte_index_at_char(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(index, _)| index)
        .unwrap_or(text.len())
}
