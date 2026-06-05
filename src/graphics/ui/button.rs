use uuid::Uuid;

use crate::{
    component::Transform,
    graphics::{render_layer, RenderCommand, RenderCommandSource, Texture},
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

    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        let image = self.image_for_state(ButtonState::Default).image();

        let (origin_x, origin_y) = image.origin.unwrap_or_default();
        let left = f64::from(self.transform.x - origin_x as f32);
        let top = f64::from(self.transform.y - origin_y as f32);
        let right = left + f64::from(self.width);
        let bottom = top + f64::from(self.height);

        x >= left && x <= right && y >= top && y <= bottom
    }

    fn image_for_state(&self, state: ButtonState) -> &ImageHandle {
        self.images[state as usize].as_ref().unwrap_or(
            self.images[ButtonState::Default as usize]
                .as_ref()
                .expect("button should have a default image"),
        )
    }
}

impl RenderCommandSource for Button {
    fn append_render_commands(&self, commands: &mut Vec<RenderCommand>) {
        let image = self.image_for_state(self.state);

        commands.push(RenderCommand {
            id: self.id,
            texture: Texture::from_image(image.image()),
            transform: self.transform,
            layer: render_layer(self.transform.z),
            camera_affected: false,
        });
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
