pub struct Button {
    pub width: u32,
    pub height: u32,
    state: ButtonState,
    pub on_click: fn(),
}

impl Button {
    pub fn new(width: u32, height: u32, on_click: fn()) -> Self {
        Self {
            width,
            height,
            state: ButtonState::Normal,
            on_click,
        }
    }
}

pub enum ButtonState {
    Normal,
    Pressed,
    MouseOver,
    Disabled,
}
