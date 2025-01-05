use winit::event::MouseButton;

use crate::{
    graphics::{button::ButtonState, RenderableV2},
    state::State,
};

use super::System;

#[derive(Default)]
pub struct ButtonSystem;

impl System for ButtonSystem {
    fn execute(&self, state: &mut State) {
        let (mouse_x, mouse_y) = state.cursor().position();
        let is_clicking = state.cursor().is_button_pressed(MouseButton::Left);

        for button in state.buttons.iter_mut() {
            let transform = button.transform();

            // The mouse is currently hovering over the button.
            if mouse_x >= transform.x.into()
                && mouse_x <= (transform.x + button.width as f32).into()
                && mouse_y >= transform.y.into()
                && mouse_y <= (transform.y + button.height as f32).into()
            {
                if is_clicking {
                    button.state = ButtonState::Pressed;

                    // FIXME: this clicks multiple times.
                    if button.on_click.is_some() {
                        (button.on_click.unwrap())();
                    }
                } else {
                    button.state = ButtonState::Hovered;
                }
            } else {
                button.state = ButtonState::Default;
            }
        }
    }
}
