use crate::{component::Transform, graphics::Sprite, state::State};

// FIXME: we need a better way for layering, currently items at the top of this list
// are rendered on top.
// TODO: i think these bitmaps have some pivot info we need to consider.
const BITMAPS: &[(&'static str, f32, f32, f32)] = &[
    // Main background
    ("Map001.nx/Back/login.img/back/11", 400.0, 300.0, 1.0),
    // Login signboard
    ("UI.nx/Login.img/Title/signboard", 391.0, 330.0, 1.0),
    // Border around login screen
    // (NxFileType::Ui, "Login.img/Common/frame"),
    // Background side trees
    // (NxFileType::Map001, "Back/login.img/back/35"),
];

pub trait Scene {
    fn init(&mut self, world: &mut State) {}
}

#[derive(Default)]
pub struct LoginScene {}

impl Scene for LoginScene {
    fn init(&mut self, state: &mut State) {
        for (path, x, y, z) in BITMAPS.iter() {
            state.spawn((Sprite::new(path), Transform::from_xyz(*x, *y, *z)));
        }
    }
}
