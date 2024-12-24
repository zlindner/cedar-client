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

            // TODO: it seems like getting the bitmap is very slow, might need to cache.
            // we might also want to move bitmap loading to the rendering thread.
            let assets = state.assets();
            let texture = assets.get_texture(&sprite.texture_path).unwrap();
            // log::info!("{}: {:?}", &sprite.texture_path, texture);

            // FIXME: this updates the transform uniforms regardless if sprite/camera doesn't move.
            let uniform = Uniform::compute(transform, &camera, &texture);
            updates.push(RenderUpdate::UpdateTransformUniform { entity, uniform });

            if !self.initialized_entities.contains(&entity) {
                updates.push(RenderUpdate::CreateIndexBuffer {
                    entity,
                    data: sprite.get_index_buffer().to_vec(),
                });

                updates.push(RenderUpdate::CreateVertexBuffer {
                    entity,
                    data: sprite.get_vertex_buffer(&texture).to_vec(),
                });

                self.initialized_entities.insert(entity);
            }

            if !self.initialized_bitmaps.contains(&sprite.texture_path) {
                // We need to push the bind group update last since it consumes the bitmap.
                updates.push(RenderUpdate::CreateTextureBindGroup {
                    texture_path: sprite.texture_path.clone(),
                    texture,
                });

                self.initialized_bitmaps.insert(sprite.texture_path.clone());
            }
        }

        updates
    }

    fn get_render_items(&mut self, state: &mut State) -> Vec<RenderItem> {
        let mut items = Vec::new();

        for (entity, (sprite, transform)) in state.query::<(&Sprite, &Transform)>().iter() {
            items.push(RenderItem {
                entity,
                type_name: std::any::type_name::<Sprite>().to_string(),
                texture_name: Some(sprite.texture_path.clone()),
                range: sprite.index_buffer_range(),
                layer: transform.z as usize,
            });
        }

        // Sort render items by their z position/layer.
        // High layer = front, low layer = back.
        items.sort_by(|a, b| b.layer.cmp(&a.layer));
        items
    }
}
