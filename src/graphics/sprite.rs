use uuid::Uuid;

use crate::{component::Transform, resource::ImageHandle};

use super::{render_layer, RenderCommand, RenderCommandSource, Texture};

// TODO: not a fan of this being in graphics, more like a game component.
pub struct Sprite {
    id: Uuid,
    image: ImageHandle,
    transform: Transform,
    camera_affected: bool,
}

impl Sprite {
    pub fn new(image: ImageHandle) -> Self {
        Self {
            id: Uuid::new_v4(),
            image,
            transform: Transform::default(),
            camera_affected: true,
        }
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    pub fn transform(&self) -> Transform {
        self.transform
    }

    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }

    pub fn with_screen_space(mut self) -> Self {
        self.camera_affected = false;
        self
    }
}

impl RenderCommandSource for Sprite {
    fn append_render_commands(&self, commands: &mut Vec<RenderCommand>) {
        commands.push(RenderCommand {
            id: self.id,
            texture: Texture::from_image(self.image.image()),
            transform: self.transform,
            layer: render_layer(self.transform.z),
            camera_affected: self.camera_affected,
        });
    }
}
