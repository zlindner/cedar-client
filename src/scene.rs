use crate::{
    component::Transform,
    graphics::{Button, Texture},
    state::State,
};

// TODO: we can maybe have "background", "foreground", and "UI" z instead of aribtraty values.
const SPRITES: &[(&'static str, f32, f32, f32)] = &[
    // Main background
    ("Map001.nx/Back/login.img/back/11", 400.0, 300.0, 1.0),
    // Login signboard
    ("UI.nx/Login.img/Title/signboard", 391.0, 330.0, 10.0),
    // Border around login screen
    ("UI.nx/Login.img/Common/frame", 400.0, 300.0, 10.0),
    // Background side trees
    ("Map001.nx/Back/login.img/back/35", 399.0, 260.0, 2.0),
    // "Save loginID" checkbox
    (
        "UI.nx/Login.img/Title/BtLoginIDSave/normal/0",
        303.0,
        332.0,
        11.0,
    ),
    // "Find loginID" checkbox
    (
        "UI.nx/Login.img/Title/BtLoginIDLost/normal/0",
        375.0,
        332.0,
        11.0,
    ),
    // "Find P/W" checkbox
    (
        "UI.nx/Login.img/Title/BtPasswdLost/normal/0",
        447.0,
        332.0,
        11.0,
    ),
    // "Join" button
    ("UI.nx/Login.img/Title/BtNew/normal/0", 291.0, 352.0, 11.0),
    // "Website" button
    (
        "UI.nx/Login.img/Title/BtHomePage/normal/0",
        363.0,
        352.0,
        11.0,
    ),
    // "Exit" button
    ("UI.nx/Login.img/Title/BtQuit/normal/0", 435.0, 352.0, 11.0),
];

pub trait Scene {
    fn init(&mut self, state: &mut State) {}
}

#[derive(Default)]
pub struct LoginScene;

impl Scene for LoginScene {
    fn init(&mut self, state: &mut State) {
        for (path, x, y, z) in SPRITES.iter() {
            let assets = state.assets();
            let texture = assets.get_texture(path).unwrap();
            drop(assets);

            state.spawn((texture, Transform::from_xyz(*x, *y, *z)));
        }

        // "Login" button
        let (button, texture) = get_button("UI.nx/Login.img/Title/BtLogin", state);
        state.spawn((button, texture, Transform::from_xyz(454.0, 279.0, 11.0)));
    }
}

fn get_button(base_path: &str, state: &mut State) -> (Button, Texture) {
    let assets = state.assets();
    let texture = assets
        .get_texture(&format!("{}/normal/0", base_path))
        .unwrap();

    (
        Button::new(texture.width, texture.height, || log::info!("Clicked!")),
        texture,
    )
}
