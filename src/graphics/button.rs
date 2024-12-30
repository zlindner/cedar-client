pub struct Button {
    width: u32,
    height: u32,
    state: ButtonState,
}

impl Button {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            state: ButtonState::Normal,
        }
    }
}

pub enum ButtonState {
    Normal,
    Pressed,
    MouseOver,
    Disabled,
}
