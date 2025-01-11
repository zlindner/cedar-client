use std::{collections::HashSet, sync::mpsc};

use uuid::Uuid;

use crate::{component::Camera, state::State};

use super::{renderer::RenderUpdate, RenderItem, RenderableV2, RendererEvent, Texture, Uniform};

// TODO: kinda hate this name...
pub struct RendererManager {
    sender: mpsc::Sender<RendererEvent>,
    initialized_entities: HashSet<Uuid>,
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

        for sprite in state.sprites.iter() {
            updates.append(&mut self.get_updates_for_component(sprite, &camera));
        }

        for button in state.buttons.iter() {
            updates.append(&mut self.get_updates_for_component(button, &camera));
        }

        for text in state.text.iter() {
            updates.append(&mut self.get_updates_for_component(text, &camera));
        }

        updates
    }

    fn get_render_items(&mut self, state: &mut State) -> Vec<RenderItem> {
        let mut items = Vec::new();

        for sprite in state.sprites.iter() {
            items.push(RenderItem {
                id: *sprite.id(),
                type_name: std::any::type_name::<Texture>().to_string(),
                texture_name: Some(sprite.texture().path.clone()),
                range: sprite.texture().index_buffer_range.clone(),
                layer: sprite.transform().z as usize,
            });
        }

        for button in state.buttons.iter() {
            items.push(RenderItem {
                id: *button.id(),
                type_name: std::any::type_name::<Texture>().to_string(),
                texture_name: Some(button.texture().path.clone()),
                range: button.texture().index_buffer_range.clone(),
                layer: button.transform().z as usize,
            });
        }

        for text in state.text.iter() {
            items.push(RenderItem {
                id: *text.id(),
                type_name: std::any::type_name::<Texture>().to_string(),
                texture_name: Some(text.texture().path.clone()),
                range: text.texture().index_buffer_range.clone(),
                layer: text.transform().z as usize,
            });
        }

        // Sort render items by their z position/layer.
        // High layer = front, low layer = back.
        // TODO: instead of this we should have a RenderLayer enum, ex. UI, Foreground, Background, ...
        items.sort_by(|a, b| b.layer.cmp(&a.layer));
        items
    }

    fn get_updates_for_component<T: RenderableV2>(
        &mut self,
        component: &T,
        camera: &Camera,
    ) -> Vec<RenderUpdate> {
        let mut updates = Vec::new();

        let id = component.id();
        let texture = component.texture();
        let transform = component.transform();

        if !self.initialized_entities.contains(id) {
            updates.push(RenderUpdate::CreateIndexBuffer {
                id: *id,
                data: texture.index_buffer.clone(),
            });

            updates.push(RenderUpdate::CreateVertexBuffer {
                id: *id,
                data: texture.vertex_buffer.clone(),
            });

            self.initialized_entities.insert(*id);
        }

        if !self.initialized_bitmaps.contains(&texture.path) {
            self.initialized_bitmaps.insert(texture.path.clone());

            updates.push(RenderUpdate::CreateTextureBindGroup {
                path: texture.path.clone(),
                width: texture.width,
                height: texture.height,
                data: texture.data.clone(),
            });
        }

        let uniform = Uniform::compute(texture, transform, camera);
        updates.push(RenderUpdate::UpdateTransformUniform { id: *id, uniform });

        updates
    }
}
