use nx_pkg4::node::Node;

use crate::{graphics::Sprite, resource::NxFileType, state::State};

// FIXME: we need a better way for layering, currently items at the top of this list
// are rendered on top.
const BITMAPS: &[(NxFileType, &'static str, f32, f32)] = &[
    (NxFileType::Ui, "Login.img/Title/signboard", 391.0, 330.0),
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
        let mut assets = state.assets_mut();

        for (file_type, path, x, y) in BITMAPS.iter() {
            match assets.nx(file_type).get(path).bitmap() {
                Ok(Some(bitmap)) => {
                    assets.register_bitmap(path, bitmap);
                }
                Ok(None) => {
                    log::warn!("Error registering bitmap {}: not found", path);
                }
                Err(e) => {
                    log::error!("Error registering bitmap {}: {}", path, e);
                }
            };
        }

        // TODO: this is bad.
        drop(assets);

        for (file_type, path, x, y) in BITMAPS.iter() {
            state.spawn((Sprite::new(path.to_string()), (x, y)));
        }
    }
}
