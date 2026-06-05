use std::collections::{HashMap, HashSet};

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
    transform_uniforms: HashMap<Uuid, Uniform>,
}

impl RenderQueueBuilder {
    pub fn new(sender: EventLoopProxy<RendererEvent>) -> Self {
        Self {
            sender,
            initialized_entities: HashSet::new(),
            initialized_textures: HashSet::new(),
            transform_uniforms: HashMap::new(),
        }
    }

    pub fn generate_and_send_events(&mut self, state: &mut State) {
        let commands = self.get_render_commands(state);
        let camera = state.get_resource::<Camera>().unwrap();
        let screen_camera = camera.screen_space();
        let visible_commands = commands
            .iter()
            .filter(|command| {
                let command_camera = command_camera(command, &camera, &screen_camera);
                command_intersects_camera(command, command_camera)
            })
            .collect::<Vec<_>>();
        let updates = self.get_render_updates(&visible_commands, &camera, &screen_camera);
        let items = self.get_render_items(&visible_commands);

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

        for sprite in &state.sprites {
            sprite.append_render_commands(&mut commands);
        }

        for button in &state.buttons {
            button.append_render_commands(&mut commands);
        }

        for text_input in &state.text_inputs {
            text_input.append_render_commands(&mut commands);
        }

        commands
    }

    fn get_render_updates(
        &mut self,
        commands: &[&RenderCommand],
        camera: &Camera,
        screen_camera: &Camera,
    ) -> Vec<RenderUpdate> {
        let mut updates = Vec::new();

        for command in commands {
            let command_camera = command_camera(command, camera, screen_camera);
            updates.append(&mut self.get_updates_for_command(command, command_camera));
        }

        updates
    }

    fn get_render_items(&mut self, commands: &[&RenderCommand]) -> Vec<RenderItem> {
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
        if self.transform_uniforms.get(&id) != Some(&uniform) {
            self.transform_uniforms.insert(id, uniform);
            updates.push(RenderUpdate::UpdateTransformUniform { id, uniform });
        }

        updates
    }
}

fn command_intersects_camera(command: &RenderCommand, camera: &Camera) -> bool {
    let (origin_x, origin_y) = command.texture.image.origin.unwrap_or_default();
    let scale = command.transform.scale;
    let left = command.transform.x - origin_x as f32 * scale;
    let top = command.transform.y - origin_y as f32 * scale;
    let right = left + command.texture.image.width as f32 * scale;
    let bottom = top + command.texture.image.height as f32 * scale;

    let camera_left = camera.left;
    let camera_top = camera.top;
    let camera_right = camera.right;
    let camera_bottom = -camera.bottom;

    right >= camera_left && left <= camera_right && bottom >= camera_top && top <= camera_bottom
}

fn command_camera<'a>(
    command: &RenderCommand,
    world_camera: &'a Camera,
    screen_camera: &'a Camera,
) -> &'a Camera {
    if command.camera_affected {
        world_camera
    } else {
        screen_camera
    }
}
