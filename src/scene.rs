use crate::{
    component::{Colour, Transform},
    graphics::{
        ui::{load_button_images, Button, TextInput},
        Sprite,
    },
    resource::{AssetManager, FontDescriptor},
    state::State,
};

pub trait Scene {
    fn init(&mut self, _state: &mut State) {}
}

#[derive(Default)]
pub struct LoginScene;

impl Scene for LoginScene {
    fn init(&mut self, state: &mut State) {
        init_sprites(state);
        init_buttons(state);
        init_text_inputs(state);
    }
}

// TODO: we might eventually want sprites to be more complex (animations, hiding, etc.), so we may
// want to create a simple "UiImage" struct or something for these.
fn init_sprites(state: &mut State) {
    let sprites = {
        let mut assets = state
            .get_resource_mut::<AssetManager>()
            .expect("AssetManager should exist");

        vec![
            Sprite::new(
                assets
                    .load_image("Map001.nx/Back/login.img/back/11")
                    .unwrap(),
            )
            .with_transform(Transform::from_xyz(400.0, 300.0, 1.0)),
            Sprite::new(
                assets
                    .load_image("Map001.nx/Back/login.img/back/35")
                    .unwrap(),
            )
            .with_transform(Transform::from_xyz(399.0, 260.0, 2.0)),
            Sprite::new(
                assets
                    .load_image("MapPretty.nx/Back/login.img/ani/16/0")
                    .unwrap(),
            )
            .with_transform(Transform::from_xyz(394.0, 173.0, 2.0)),
            Sprite::new(
                assets
                    .load_image("UI.nx/Login.img/Title/signboard")
                    .unwrap(),
            )
            .with_transform(Transform::from_xyz(391.0, 330.0, 10.0)),
            Sprite::new(assets.load_image("UI.nx/Login.img/Common/frame").unwrap())
                .with_transform(Transform::from_xyz(400.0, 300.0, 10.0)),
        ]
    };

    state.sprites.extend(sprites);
}

fn init_buttons(state: &mut State) {
    let buttons = {
        let mut assets = state
            .get_resource_mut::<AssetManager>()
            .expect("AssetManager should exist");

        vec![
            Button::new(load_button_images(
                &mut assets,
                "UI.nx/Login.img/Title/BtLogin",
            ))
            .with_transform(Transform::from_xyz(454.0, 279.0, 11.0))
            .with_on_click(|| log::info!("login")),
            // TODO: is this supposed to be a checkbox?
            Button::new(load_button_images(
                &mut assets,
                "UI.nx/Login.img/Title/BtLoginIDSave",
            ))
            .with_transform(Transform::from_xyz(303.0, 332.0, 11.0))
            .with_on_click(|| log::info!("save_login_id")),
            Button::new(load_button_images(
                &mut assets,
                "UI.nx/Login.img/Title/BtLoginIDLost",
            ))
            .with_transform(Transform::from_xyz(375.0, 332.0, 11.0))
            .with_on_click(|| log::info!("find_login_id")),
            Button::new(load_button_images(
                &mut assets,
                "UI.nx/Login.img/Title/BtPasswdLost",
            ))
            .with_transform(Transform::from_xyz(447.0, 332.0, 11.0))
            .with_on_click(|| log::info!("find_password")),
            Button::new(load_button_images(
                &mut assets,
                "UI.nx/Login.img/Title/BtNew",
            ))
            .with_transform(Transform::from_xyz(291.0, 352.0, 11.0))
            .with_on_click(|| log::info!("join")),
            Button::new(load_button_images(
                &mut assets,
                "UI.nx/Login.img/Title/BtHomePage",
            ))
            .with_transform(Transform::from_xyz(363.0, 352.0, 11.0))
            .with_on_click(|| log::info!("website")),
            Button::new(load_button_images(
                &mut assets,
                "UI.nx/Login.img/Title/BtQuit",
            ))
            .with_transform(Transform::from_xyz(435.0, 352.0, 11.0))
            .with_on_click(|| std::process::exit(0)),
        ]
    };

    state.buttons.extend(buttons);
}

fn init_text_inputs(state: &mut State) {
    let username_input = TextInput::new(150, 24)
        .with_font(FontDescriptor::new("Arial", 13, Colour::white()))
        .with_transform(Transform::from_xyz(296.0, 279.0, 11.0));

    state.text_inputs.push(username_input);
}
