use winit::event::MouseButton;

use crate::{
    component::Transform,
    graphics::{button::ButtonState, Button, Texture},
    state::State,
};

use super::System;

#[derive(Default)]
pub struct ButtonSystem;

impl System for ButtonSystem {
    fn execute(&self, state: &mut State) {
        let (mouse_x, mouse_y) = state.cursor().position();
        let is_clicking = state.cursor().is_button_pressed(MouseButton::Left);

        for (entity, (button, transform)) in state.query_mut::<(&mut Button, &Transform)>() {
            // The mouse is currently hovering over the button.
            if mouse_x >= transform.x.into()
                && mouse_x <= (transform.x + button.width as f32).into()
                && mouse_y >= transform.y.into()
                && mouse_y <= (transform.y + button.height as f32).into()
            {
                // TODO: update the button state.

                // FIXME this clicks multiple times.
                if is_clicking {
                    button.state = ButtonState::Pressed;
                    (button.on_click)();
                }
            }
        }
    }
}
