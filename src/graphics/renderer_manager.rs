use std::{collections::HashSet, sync::mpsc};

use hecs::Entity;

use crate::{
    component::{Camera, Transform},
    state::State,
};

use super::{
    renderer::RenderUpdate, sprite::Renderable, RenderItem, RendererEvent, Sprite, Uniform,
};

// TODO: kinda hate this name...
pub struct RendererManager {
    sender: mpsc::Sender<RendererEvent>,
    initialized_entities: HashSet<Entity>,
    initialized_bitmaps: HashSet<String>,
}

impl RendererManager {
    pub fn new(sender: mpsc::Sender<RendererEvent>) -> Self {
        Self {
            sender,
            initialized_entities: HashSet::new(),
            initialized_bitmaps: HashSet::new(),
        }
    }

    pub fn generate_and_send_events(&mut self, state: &mut State) {
        let updates = self.get_render_updates(state);
        let items = self.get_render_items(state);

        if let Err(e) = self.sender.send(RendererEvent::Render(updates, items)) {
            log::error!("Error sending Render event: {}", e);
        }
    }

    fn get_render_updates(&mut self, state: &mut State) -> Vec<RenderUpdate> {
        let mut updates = Vec::new();

        let camera = state.get_resource::<Camera>().unwrap();

        for (entity, (sprite, transform)) in state.query::<(&Sprite, &Transform)>().iter() {
            let mut sprite = sprite.clone();

            if !self.initialized_entities.contains(&entity) {
                let assets = state.assets();

                // TODO: it seems like getting the bitmap is very slow, might need to cache.
                let bitmap = assets.get_bitmap(&sprite.bitmap_path).unwrap();

                updates.push(RenderUpdate::CreateIndexBuffer {
                    entity,
                    data: sprite.get_index_buffer().to_vec(),
                });

                updates.push(RenderUpdate::CreateVertexBuffer {
                    entity,
                    data: sprite.get_vertex_buffer(&bitmap).to_vec(),
                });

                self.initialized_entities.insert(entity);
            }

            if !self.initialized_bitmaps.contains(&sprite.bitmap_path) {
                let assets = state.assets();
                let bitmap = assets.get_bitmap(&sprite.bitmap_path).unwrap();

                // We need to push the bind group update last since it consumes the bitmap.
                updates.push(RenderUpdate::CreateTextureBindGroup {
                    bitmap_path: sprite.bitmap_path.clone(),
                    bitmap,
                });

                self.initialized_bitmaps.insert(sprite.bitmap_path.clone());
            }

            // FIXME: this updates the transform uniforms regardless if sprite/camera doesn't move.
            let uniform = Uniform::compute(transform, &camera);
            updates.push(RenderUpdate::UpdateTransformUniform { entity, uniform });
        }

        updates
    }

    fn get_render_items(&mut self, state: &mut State) -> Vec<RenderItem> {
        let mut items = Vec::new();

        for (entity, sprite) in state.query::<&Sprite>().iter() {
            items.push(RenderItem {
                entity,
                type_name: std::any::type_name::<Sprite>().to_string(),
                texture_name: Some(sprite.bitmap_path.clone()),
                range: sprite.index_buffer_range(),
            });
        }

        items
    }
}
