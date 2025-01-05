use std::{
    collections::HashMap,
    iter,
    ops::Range,
    sync::{mpsc, Arc},
};

use uuid::Uuid;
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

use super::{Renderable, Texture, Uniform};

pub struct Renderer {
    window: Arc<Window>,
    receiver: mpsc::Receiver<RendererEvent>,

    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    transform_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group_layout: wgpu::BindGroupLayout,

    // A map of `Renderable` type names to render pipelines.
    render_pipelines: HashMap<String, wgpu::RenderPipeline>,
    vertex_buffers: HashMap<Uuid, wgpu::Buffer>,
    index_buffers: HashMap<Uuid, wgpu::Buffer>,

    transform_bind_groups: HashMap<Uuid, (wgpu::BindGroup)>,
    texture_bind_groups: HashMap<String, (wgpu::BindGroup, wgpu::Texture)>,
}

impl Renderer {
    pub async fn new(window: Arc<Window>, receiver: mpsc::Receiver<RendererEvent>) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

        let window_size = window.inner_size();
        let surface = instance
            .create_surface(window.clone())
            .expect("surface should be created");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("adapter should be created");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .expect("device and queue should be created");

        let config = surface
            .get_default_config(&adapter, window_size.width, window_size.height)
            .expect("surface configuration should be created");

        surface.configure(&device, &config);

        let transform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: None,
            });

        Self {
            window,
            receiver,
            surface,
            device,
            queue,
            config,
            transform_bind_group_layout,
            texture_bind_group_layout,
            render_pipelines: HashMap::new(),
            vertex_buffers: HashMap::new(),
            index_buffers: HashMap::new(),
            transform_bind_groups: HashMap::new(),
            texture_bind_groups: HashMap::new(),
        }
    }

    pub fn run(mut self) {
        self.register_render_pipeline::<Texture>();

        loop {
            if let Ok(event) = self.receiver.recv() {
                match event {
                    RendererEvent::Render(updates, items) => {
                        self.process_updates(updates);

                        match self.render(items) {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                self.resize(self.window.inner_size());
                            }
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                log::error!("System is out of memory, exiting");
                                // TODO: event_loop.exit();
                                // can probably use a one-shot channel for this.
                            }
                            Err(wgpu::SurfaceError::Timeout) => {
                                log::warn!("Frame took longer than expected to render");
                            }
                        }
                    }
                    RendererEvent::Resize(new_size) => self.resize(new_size),
                }
            }
        }
    }

    fn process_updates(&mut self, mut updates: Vec<RenderUpdate>) {
        while let Some(update) = updates.pop() {
            match update {
                RenderUpdate::CreateTextureBindGroup {
                    path,
                    width,
                    height,
                    data,
                } => {
                    self.register_texture(path, width, height, data);
                }
                RenderUpdate::CreateIndexBuffer { id, data } => {
                    self.index_buffers.insert(
                        id,
                        self.device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: None,
                                contents: &data,
                                usage: wgpu::BufferUsages::INDEX,
                            }),
                    );
                }
                RenderUpdate::CreateVertexBuffer { id, data } => {
                    self.vertex_buffers.insert(
                        id,
                        self.device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: None,
                                contents: &data,
                                usage: wgpu::BufferUsages::VERTEX,
                            }),
                    );
                }
                RenderUpdate::UpdateTransformUniform { id, uniform } => {
                    let uniform_buffer =
                        self.device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: None,
                                contents: bytemuck::cast_slice(&[uniform]),
                                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                            });

                    let uniform_bind_group =
                        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            layout: &self.transform_bind_group_layout,
                            entries: &[wgpu::BindGroupEntry {
                                binding: 0,
                                resource: uniform_buffer.as_entire_binding(),
                            }],
                            label: Some("uniform_bind_group"),
                        });

                    self.transform_bind_groups.insert(id, uniform_bind_group);
                    self.queue
                        .write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(&[uniform]));
                }
            }
        }
    }

    fn render(&mut self, mut items: Vec<RenderItem>) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        // Process each render item one by one.
        while let Some(item) = items.pop() {
            // Set the pipeline for the item type.
            let render_pipeline = self.render_pipelines.get(&item.type_name).unwrap();
            render_pass.set_pipeline(render_pipeline);

            // Set the vertex buffer for the item's entity.
            let vertex_buffer = self.vertex_buffers.get(&item.id).unwrap();
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));

            // Set the index buffer for the item's entity.
            let index_buffer = self.index_buffers.get(&item.id).unwrap();
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            let transform_bind_group = self.transform_bind_groups.get(&item.id).unwrap();
            render_pass.set_bind_group(0, transform_bind_group, &[]);

            // Set the bind group for the item's texture (if applicable).
            if let Some(texture_name) = item.texture_name {
                let texture_bind_group = self.texture_bind_groups.get(&texture_name).unwrap();
                render_pass.set_bind_group(1, &texture_bind_group.0, &[]);
            }

            render_pass.draw_indexed(item.range, 0, 0..1);
        }

        drop(render_pass);
        self.queue.submit(iter::once(encoder.finish()));
        frame.present();

        Ok(())
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;

            // FIXME: this only works on the main thread
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn register_render_pipeline<T>(&mut self)
    where
        T: Renderable,
    {
        self.render_pipelines.insert(
            std::any::type_name::<T>().to_string(),
            T::create_render_pipeline(
                &self.device,
                &self.transform_bind_group_layout,
                &self.texture_bind_group_layout,
                &self.config,
            ),
        );
    }

    pub fn register_texture(&mut self, path: String, width: u32, height: u32, data: Vec<u8>) {
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let wgpu_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // NxBitmap data is in the "reversed" bgra format.
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some(&path),
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &wgpu_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * texture_size.width),
                rows_per_image: Some(texture_size.height),
            },
            texture_size,
        );

        let view = wgpu_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            // TODO: play with these to see what looks best
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: None,
        });

        self.texture_bind_groups
            .insert(path, (texture_bind_group, wgpu_texture));
    }
}

pub enum RendererEvent {
    Render(Vec<RenderUpdate>, Vec<RenderItem>),
    Resize(PhysicalSize<u32>),
}

pub enum RenderUpdate {
    CreateTextureBindGroup {
        path: String,
        width: u32,
        height: u32,
        data: Vec<u8>,
    },
    CreateIndexBuffer {
        id: Uuid,
        data: Vec<u8>,
    },
    CreateVertexBuffer {
        id: Uuid,
        data: Vec<u8>,
    },
    UpdateTransformUniform {
        id: Uuid,
        uniform: Uniform,
    },
}

pub struct RenderItem {
    pub(crate) id: Uuid,
    pub(crate) type_name: String,
    pub(crate) texture_name: Option<String>,
    pub(crate) layer: usize,
    pub(crate) range: Range<u32>,
}
