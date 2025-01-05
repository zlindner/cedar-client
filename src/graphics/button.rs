use uuid::Uuid;

use crate::{component::Transform, resource::AssetManager};

use super::{RenderableV2, Texture};

// TODO: not a fan of this being in graphics, more like a game component.
pub struct Button {
    id: Uuid,

    pub width: u32,
    pub height: u32,
    pub state: ButtonState,

    textures: [Option<Texture>; 4],
    transform: Transform,

    pub on_click: Option<fn()>,
}

impl Button {
    pub fn new(nx_path: &str) -> Self {
        let textures = load_textures(nx_path);
        let default_texture = textures[ButtonState::Default as usize]
            .as_ref()
            .expect("button should have a default texture");

        Self {
            id: Uuid::new_v4(),
            width: default_texture.width,
            height: default_texture.height,
            state: ButtonState::Default,
            textures,
            transform: Transform::default(),
            on_click: None,
        }
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    pub fn with_on_click(mut self, on_click: fn()) -> Self {
        self.on_click = Some(on_click);
        self
    }
}

impl RenderableV2 for Button {
    fn id(&self) -> &Uuid {
        &self.id
    }

    fn texture(&self) -> &Texture {
        self.textures[self.state as usize].as_ref().unwrap_or(
            self.textures[ButtonState::Default as usize]
                .as_ref()
                .expect("button should have a default texture"),
        )
    }

    fn transform(&self) -> &Transform {
        &self.transform
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ButtonState {
    Default = 0,
    Pressed = 1,
    Hovered = 2,
    Disabled = 3,
}

fn load_textures(nx_path: &str) -> [Option<Texture>; 4] {
    [
        AssetManager::get_texture(&format!("{}/normal/0", nx_path)),
        AssetManager::get_texture(&format!("{}/pressed/0", nx_path)),
        AssetManager::get_texture(&format!("{}/mouseOver/0", nx_path)),
        AssetManager::get_texture(&format!("{}/disabled/0", nx_path)),
    ]
}
