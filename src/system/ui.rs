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
pub fn text_input_system(state: &mut State) {
    let (mouse_x, mouse_y, clicked) = {
        let cursor = state.cursor();
        let (mouse_x, mouse_y) = cursor.position();
        (
            mouse_x,
            mouse_y,
            cursor.was_button_pressed(MouseButton::Left),
        )
    };

    let (typed_text, backspaces, tab_pressed, enter_pressed) = {
        let keyboard = state.keyboard();
        (
            keyboard.text().to_string(),
            keyboard.backspaces(),
            keyboard.tab_pressed(),
            keyboard.enter_pressed(),
        )
    };

    if clicked {
        let focused_index = state
            .text_inputs
            .iter()
            .position(|input| input.contains(mouse_x, mouse_y));

        for (index, input) in state.text_inputs.iter_mut().enumerate() {
            input.focused = focused_index == Some(index);
        }
    }

    if tab_pressed || enter_pressed {
        focus_next_text_input(state);
    }

    let Some(focused_input) = state.text_inputs.iter_mut().find(|input| input.focused) else {
        return;
    };

    if !typed_text.is_empty() {
        focused_input.append_text(&typed_text);
    }

    for _ in 0..backspaces {
        focused_input.backspace();
    }
}

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
                input.display_text(),
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

pub fn clear_input_events_system(state: &mut State) {
    state.cursor().clear_events();
    state.keyboard().clear_events();
}

fn focus_next_text_input(state: &mut State) {
    if state.text_inputs.is_empty() {
        return;
    }

    let next_index = match state.text_inputs.iter().position(|input| input.focused) {
        Some(index) => (index + 1) % state.text_inputs.len(),
        None => 0,
    };

    for (index, input) in state.text_inputs.iter_mut().enumerate() {
        input.focused = index == next_index;
    }
}
