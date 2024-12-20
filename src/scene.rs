use crate::{graphics::Sprite, state::State};

// FIXME: we need a better way for layering, currently items at the top of this list
// are rendered on top.
const BITMAPS: &[(&'static str, f32, f32)] = &[
    ("UI.nx/Login.img/Title/signboard", 391.0, 330.0),
    // Login signboard
    // (NxFileType::Ui, "Login.img/Title/signboard"),
    // Border around login screen
    // (NxFileType::Ui, "Login.img/Common/frame"),
    // Background side trees
    // (NxFileType::Map001, "Back/login.img/back/35"),
    // Main background
    // (NxFileType::Map001, "Back/login.img/back/11"),
];

pub trait Scene {
    fn init(&mut self, world: &mut State) {}
}

#[derive(Default)]
pub struct LoginScene {}

impl Scene for LoginScene {
    fn init(&mut self, state: &mut State) {
        for (path, x, y) in BITMAPS.iter() {
            state.spawn((Sprite::new(path), (x, y)));
        }
    }
}
