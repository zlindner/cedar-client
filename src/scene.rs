use nx_pkg4::node::Node;

use crate::{ecs::World, resource::asset_manager::NxFileType};

const BITMAPS: &[(NxFileType, &'static str)] = &[(NxFileType::Map001, "Back/login.img/back/11")];

pub trait Scene {
    fn init(&mut self, world: &mut World) {}
}

#[derive(Default)]
pub struct LoginScene {}

impl Scene for LoginScene {
    fn init(&mut self, world: &mut World) {
        let mut assets = world.assets_mut();

        log::info!("Registering login bitmaps");

        for (file_type, path) in BITMAPS.iter() {
            match assets.nx(file_type).get(path).bitmap() {
                Ok(Some(bitmap)) => {
                    assets.register_bitmap(path, bitmap);
                }
                Ok(None) => {
                    log::warn!("Error registering bitmap {}: not found", path)
                }
                Err(e) => {
                    log::error!("Error registering bitmap {}: {}", path, e)
                }
            };
        }

        let x = assets
            .nx(&NxFileType::Map001)
            .get("Back")
            .get("login.img")
            .get("back")
            .get("11")
            .bitmap()
            .unwrap()
            .unwrap();

        assets.register_bitmap("test", x);
    }
}
