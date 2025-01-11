use winit::event::MouseButton;

use crate::{
    component::Transform,
    graphics::{
        ui::{ButtonState, Text},
        RenderableV2,
    },
    resource::AssetManager,
    state::State,
};

///
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

///
pub fn text_system(state: &mut State) {
    for input in state.text_inputs.iter_mut() {
        // TODO: this should be the font/font size/colour of the input
        let font = AssetManager::get_font("default").unwrap();

        if !input.changed {
            continue;
        }

        input.changed = false;

        let mut current_pos = 0.0;

        for input_character in input.text.chars() {
            if input_character.is_whitespace() {
                current_pos += 5.;
                continue;
            }

            let character = font.characters.get(&input_character).unwrap();
            let transform = Transform::from_xyz(
                current_pos,
                font.compute_vertical_offset(character.y.0),
                20.0,
            );

            // TODO: append any x/y padding from input

            current_pos = current_pos + character.width + 2.;

            let ui_text = Text::new(character, font).with_transform(transform);
            log::info!("{:?}", ui_text);
            state.text.push(ui_text);
        }
    }
}
