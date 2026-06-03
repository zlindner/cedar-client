use uuid::Uuid;

use crate::{
    component::Transform,
    graphics::{RenderableV2, Texture},
    resource::{AssetManager, ImageHandle},
};

// TODO: not a fan of this being in graphics, more like a game component.
pub struct Button {
    id: Uuid,

    pub width: u32,
    pub height: u32,
    pub state: ButtonState,

    images: [Option<ImageHandle>; 4],
    transform: Transform,

    pub on_click: Option<fn()>,
}

impl Button {
    pub fn new(images: [Option<ImageHandle>; 4]) -> Self {
        let default_image = images[ButtonState::Default as usize]
            .as_ref()
            .map(ImageHandle::image)
            .expect("button should have a default image");

        Self {
            id: Uuid::new_v4(),
            width: default_image.width,
            height: default_image.height,
            state: ButtonState::Default,
            images,
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

    fn texture(&self) -> Texture {
        let image = self.images[self.state as usize].as_ref().unwrap_or(
            self.images[ButtonState::Default as usize]
                .as_ref()
                .expect("button should have a default image"),
        );

        Texture::from_image(image.image())
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

pub fn load_button_images(assets: &mut AssetManager, nx_path: &str) -> [Option<ImageHandle>; 4] {
    [
        assets.load_image(&format!("{}/normal/0", nx_path)),
        assets.load_image(&format!("{}/pressed/0", nx_path)),
        assets.load_image(&format!("{}/mouseOver/0", nx_path)),
        assets.load_image(&format!("{}/disabled/0", nx_path)),
    ]
}
