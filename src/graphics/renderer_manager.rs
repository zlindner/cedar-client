use std::sync::mpsc;

use crate::state::State;

use super::{renderer::RenderUpdate, sprite::Renderable, RenderItem, RendererEvent, Sprite};

// TODO: kinda hate this name...
pub struct RendererManager {
    sender: mpsc::Sender<RendererEvent>,
}

impl RendererManager {
    pub fn new(sender: mpsc::Sender<RendererEvent>) -> Self {
        Self { sender }
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

        for (entity, sprite) in state.query_mut::<&mut Sprite>() {
            updates.push(RenderUpdate::CreateIndexBuffer {
                entity,
                data: sprite.get_index_buffer().to_vec(),
            });

            updates.push(RenderUpdate::CreateVertexBuffer {
                entity,
                data: sprite.get_vertex_buffer().to_vec(),
            });
        }

        updates
    }

    fn get_render_items(&mut self, state: &mut State) -> Vec<RenderItem> {
        let mut items = Vec::new();

        for (entity, sprite) in state.query_mut::<&mut Sprite>() {
            let s = sprite.clone();

            items.push(RenderItem {
                entity,
                type_name: std::any::type_name::<Sprite>().to_string(),
                texture_name: Some(s.texture_path),
                range: sprite.index_buffer_range(),
            });
        }

        items
    }
}
