use crate::{
    component::Transform,
    graphics::{Button, Sprite},
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
    }
}

// TODO: we might eventually want sprites to be more complex (animations, hiding, etc.), so we may
// want to create a simple "UiImage" struct or something for these.
fn init_sprites(state: &mut State) {
    let main_background = Sprite::new("Map001.nx/Back/login.img/back/11")
        .with_transform(Transform::from_xyz(400.0, 300.0, 1.0));

    let side_trees = Sprite::new("Map001.nx/Back/login.img/back/35")
        .with_transform(Transform::from_xyz(399.0, 260.0, 2.0));

    let maplestory_logo = Sprite::new("MapPretty.nx/Back/login.img/ani/16/0")
        .with_transform(Transform::from_xyz(394.0, 173.0, 2.0));

    let signboard = Sprite::new("UI.nx/Login.img/Title/signboard")
        .with_transform(Transform::from_xyz(391.0, 330.0, 10.0));

    let border = Sprite::new("UI.nx/Login.img/Common/frame")
        .with_transform(Transform::from_xyz(400.0, 300.0, 10.0));

    state.sprites.push(main_background);
    state.sprites.push(side_trees);
    state.sprites.push(maplestory_logo);
    state.sprites.push(signboard);
    state.sprites.push(border);
}

fn init_buttons(state: &mut State) {
    let login_button = Button::new("UI.nx/Login.img/Title/BtLogin")
        .with_transform(Transform::from_xyz(454.0, 279.0, 11.0))
        .with_on_click(|| log::info!("login"));

    // TODO: is this supposed to be a checkbox?
    let save_login_id_button = Button::new("UI.nx/Login.img/Title/BtLoginIDSave")
        .with_transform(Transform::from_xyz(303.0, 332.0, 11.0))
        .with_on_click(|| log::info!("save_login_id"));

    let find_login_id_button = Button::new("UI.nx/Login.img/Title/BtLoginIDLost")
        .with_transform(Transform::from_xyz(375.0, 332.0, 11.0))
        .with_on_click(|| log::info!("find_login_id"));

    let find_password_button = Button::new("UI.nx/Login.img/Title/BtPasswdLost")
        .with_transform(Transform::from_xyz(447.0, 332.0, 11.0))
        .with_on_click(|| log::info!("find_password"));

    let join_button = Button::new("UI.nx/Login.img/Title/BtNew")
        .with_transform(Transform::from_xyz(291.0, 352.0, 11.0))
        .with_on_click(|| log::info!("join"));

    let website_button = Button::new("UI.nx/Login.img/Title/BtHomePage")
        .with_transform(Transform::from_xyz(363.0, 352.0, 11.0))
        .with_on_click(|| log::info!("website"));

    let exit_button = Button::new("UI.nx/Login.img/Title/BtQuit")
        .with_transform(Transform::from_xyz(435.0, 352.0, 11.0))
        .with_on_click(|| log::info!("exit"));

    state.buttons.push(login_button);
    state.buttons.push(save_login_id_button);
    state.buttons.push(find_login_id_button);
    state.buttons.push(find_password_button);
    state.buttons.push(join_button);
    state.buttons.push(website_button);
    state.buttons.push(exit_button);
}
