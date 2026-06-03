use winit::event::MouseButton;

use crate::{
    component::Transform,
    graphics::ui::{ButtonState, TextLayout},
    resource::{AssetManager, Font},
    state::State,
};

/// System for handling buttons - clicking, hovering, etc.
pub fn button_system(state: &mut State) {
    let (mouse_x, mouse_y) = state.cursor().position();
    let is_clicking = state.cursor().is_button_pressed(MouseButton::Left);

    for button in state.buttons.iter_mut() {
        if button.state == ButtonState::Disabled {
            continue;
        }

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

/// System for handling text inputs.
pub fn text_system(state: &mut State) {
    for index in 0..state.text_inputs.len() {
        let (font_descriptor, text, input_transform) = {
            let input = &mut state.text_inputs[index];

            // The input hasn't changed, ex. nothing was typed while focused.
            if !input.changed {
                continue;
            }

            input.changed = false;

            (
                input.font_descriptor.clone(),
                input.text.clone(),
                Transform::from_xyz(input.transform.x, input.transform.y, input.transform.z),
            )
        };

        let layout = {
            let mut assets = state
                .get_resource_mut::<AssetManager>()
                .expect("AssetManager should exist");
            // TODO: this should be the font/font size/colour of the input
            let font = assets.get_font(&font_descriptor).unwrap();

            create_text_layout(&text, &input_transform, font)
        };

        state.text_inputs[index].set_layout(layout);
    }
}

fn create_text_layout(text: &str, input_transform: &Transform, font: &Font) -> TextLayout {
    TextLayout::new(text, input_transform, font)
}
