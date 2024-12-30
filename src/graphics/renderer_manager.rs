use std::{collections::HashSet, sync::mpsc};

use hecs::Entity;

use crate::{
    component::{Camera, Transform},
    state::State,
};

use super::{renderer::RenderUpdate, RenderItem, RendererEvent, Texture, Uniform};

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

        // TODO we can probably just send a single vec, push updates first, then items.
        if let Err(e) = self.sender.send(RendererEvent::Render(updates, items)) {
            log::error!("Error sending Render event: {}", e);
        }
    }

    fn get_render_updates(&mut self, state: &mut State) -> Vec<RenderUpdate> {
        let mut updates = Vec::new();

        let camera = state.get_resource::<Camera>().unwrap();

        for (entity, (texture, transform)) in state.query::<(&Texture, &Transform)>().iter() {
            // FIXME: this updates the transform uniforms regardless if sprite/camera doesn't move.
            let uniform = Uniform::compute(transform, &camera, texture);
            updates.push(RenderUpdate::UpdateTransformUniform { entity, uniform });

            if !self.initialized_entities.contains(&entity) {
                updates.push(RenderUpdate::CreateIndexBuffer {
                    entity,
                    data: texture.index_buffer.clone(),
                });

                updates.push(RenderUpdate::CreateVertexBuffer {
                    entity,
                    data: texture.vertex_buffer.clone(),
                });

                self.initialized_entities.insert(entity);
            }

            if !self.initialized_bitmaps.contains(&texture.path) {
                self.initialized_bitmaps.insert(texture.path.clone());

                // TODO: these clones are unfortunate, not sure if better way to do this.
                updates.push(RenderUpdate::CreateTextureBindGroup {
                    path: texture.path.clone(),
                    width: texture.width,
                    height: texture.height,
                    data: texture.data.clone(),
                });
            }
        }

        updates
    }

    fn get_render_items(&mut self, state: &mut State) -> Vec<RenderItem> {
        let mut items = Vec::new();

        for (entity, (texture, transform)) in state.query::<(&Texture, &Transform)>().iter() {
            items.push(RenderItem {
                entity,
                type_name: std::any::type_name::<Texture>().to_string(),
                texture_name: Some(texture.path.clone()),
                range: texture.index_buffer_range.clone(),
                layer: transform.z as usize,
            });
        }

        // Sort render items by their z position/layer.
        // High layer = front, low layer = back.
        items.sort_by(|a, b| b.layer.cmp(&a.layer));
        items
    }
}
