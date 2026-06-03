use std::collections::HashSet;

use uuid::Uuid;
use winit::event_loop::EventLoopProxy;

use crate::{component::Camera, state::State};

use super::{
    renderer::RenderUpdate, RenderCommand, RenderCommandSource, RenderItem, RendererEvent, Texture,
    Uniform,
};

pub struct RenderQueueBuilder {
    sender: EventLoopProxy<RendererEvent>,
    initialized_entities: HashSet<Uuid>,
    initialized_textures: HashSet<String>,
}

impl RenderQueueBuilder {
    pub fn new(sender: EventLoopProxy<RendererEvent>) -> Self {
        Self {
            sender,
            initialized_entities: HashSet::new(),
            initialized_textures: HashSet::new(),
        }
    }

    pub fn generate_and_send_events(&mut self, state: &mut State) {
        let commands = self.get_render_commands(state);
        let updates = self.get_render_updates(&commands, state);
        let items = self.get_render_items(&commands);

        // TODO we can probably just send a single vec, push updates first, then items.
        if let Err(e) = self
            .sender
            .send_event(RendererEvent::Render(updates, items))
        {
            log::error!("Error sending Render event: {}", e);
        }
    }

    fn get_render_commands(&self, state: &State) -> Vec<RenderCommand> {
        let mut commands = Vec::new();

        commands.extend(state.sprites.iter().map(|sprite| sprite.render_command()));
        commands.extend(state.buttons.iter().map(|button| button.render_command()));
        commands.extend(state.text.iter().map(|text| text.render_command()));

        commands
    }

    fn get_render_updates(
        &mut self,
        commands: &[RenderCommand],
        state: &mut State,
    ) -> Vec<RenderUpdate> {
        let mut updates = Vec::new();

        let camera = state.get_resource::<Camera>().unwrap();

        for command in commands {
            updates.append(&mut self.get_updates_for_command(command, &camera));
        }

        updates
    }

    fn get_render_items(&mut self, commands: &[RenderCommand]) -> Vec<RenderItem> {
        let mut items = commands
            .iter()
            .map(|command| RenderItem {
                id: command.id,
                type_name: std::any::type_name::<Texture>().to_string(),
                texture_name: Some(command.texture.image.path.clone()),
                range: command.texture.quad.index_buffer_range.clone(),
                layer: command.layer,
            })
            .collect::<Vec<_>>();

        // Sort render items by their z position/layer.
        // High layer = front, low layer = back.
        // TODO: instead of this we should have a RenderLayer enum, ex. UI, Foreground, Background, ...
        items.sort_by(|a, b| b.layer.cmp(&a.layer));
        items
    }

    fn get_updates_for_command(
        &mut self,
        command: &RenderCommand,
        camera: &Camera,
    ) -> Vec<RenderUpdate> {
        let mut updates = Vec::new();

        let id = command.id;
        let texture = &command.texture;
        let transform = &command.transform;

        if !self.initialized_entities.contains(&id) {
            updates.push(RenderUpdate::CreateIndexBuffer {
                id,
                data: texture.quad.index_buffer.clone(),
            });

            updates.push(RenderUpdate::CreateVertexBuffer {
                id,
                data: texture.quad.vertex_buffer.clone(),
            });

            self.initialized_entities.insert(id);
        }

        if !self.initialized_textures.contains(&texture.image.path) {
            self.initialized_textures.insert(texture.image.path.clone());

            updates.push(RenderUpdate::CreateTextureBindGroup {
                path: texture.image.path.clone(),
                width: texture.image.width,
                height: texture.image.height,
                data: texture.image.data.clone(),
            });
        }

        let uniform = Uniform::compute(texture, transform, camera);
        updates.push(RenderUpdate::UpdateTransformUniform { id, uniform });

        updates
    }
}
