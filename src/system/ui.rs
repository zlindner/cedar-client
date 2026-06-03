use std::sync::Arc;

use winit::event::MouseButton;

use crate::{
    component::Transform,
    graphics::{
        ui::{ButtonState, Text},
        ImageAsset,
    },
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

        let text_items = {
            let mut assets = state
                .get_resource_mut::<AssetManager>()
                .expect("AssetManager should exist");
            // TODO: this should be the font/font size/colour of the input
            let font = assets.get_font(&font_descriptor).unwrap();

            create_text_items(&text, &input_transform, font)
        };

        state.text_inputs[index].set_glyphs(text_items);
    }
}

fn create_text_items(text: &str, input_transform: &Transform, font: &Font) -> Vec<Text> {
    let mut text_items = Vec::new();
    let atlas = Arc::new(ImageAsset::font(font));
    // TODO: we should move all of this logic to the renderer manager.
    // this system should really only handle updating the input's text, focus, etc.
    let mut current_pos = 0.0;

    for input_character in text.chars() {
        if input_character.is_whitespace() {
            // TODO: this should be based on font size.
            // white_space_size
            current_pos += 5.0;
            continue;
        }

        let Some(character) = font.characters.get(&input_character) else {
            current_pos += 5.0;
            continue;
        };
        let transform = Transform::from_xyz(
            input_transform.x + current_pos,
            input_transform.y + font.compute_vertical_offset(character.y.0),
            input_transform.z,
        );

        // TODO: append any x/y padding from input

        // TODO: this should be based on font size (size_between_char)
        current_pos = current_pos + character.width + 2.0;

        let ui_text = Text::new(character, atlas.clone()).with_transform(transform);

        // I'm thinking there should be some shared "text" struct/component that is rendered.
        // the text component should be able to be rendered by text inputs, and static text (player names, etc.)
        text_items.push(ui_text);
    }

    text_items
}
