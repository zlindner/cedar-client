use std::{collections::HashMap, iter, sync::Arc};

use nx_pkg4::NxBitmap;
use wgpu::{include_wgsl, util::DeviceExt};
use winit::{dpi::PhysicalSize, window::Window};

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    texture_bind_group_layout: wgpu::BindGroupLayout,

    // A map of `RenderItem` type names to render pipelines.
    render_pipelines: HashMap<String, wgpu::RenderPipeline>,
    vertex_buffers: HashMap<String, wgpu::Buffer>,
    index_buffers: HashMap<String, wgpu::Buffer>,

    // TODO: we might want to make this a HashMap<String, HashMap<String, (BindGroup, Texture)>>
    // and separate by render item type to ensure no collisions?
    bitmap_bind_groups: HashMap<String, (wgpu::BindGroup, wgpu::Texture)>,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

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

        let window_size = window.inner_size();

        let config = surface
            .get_default_config(&adapter, window_size.width, window_size.height)
            .expect("surface configuration should be created");

        surface.configure(&device, &config);

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
            surface,
            device,
            queue,
            config,
            texture_bind_group_layout,
            render_pipelines: HashMap::new(),
            vertex_buffers: HashMap::new(),
            index_buffers: HashMap::new(),
            bitmap_bind_groups: HashMap::new(),
        }
    }

    pub fn init(&mut self) {
        BitmapRenderItem::create_renderer_components(self);
    }

    pub fn render(&mut self, mut items: Vec<RenderItem>) -> Result<(), wgpu::SurfaceError> {
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
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
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
            let render_pipeline = self.render_pipelines.get(item.get_type_name()).unwrap();
            render_pass.set_pipeline(render_pipeline);

            // Set the vertex buffer for the item type.
            let vertex_buffer = self.vertex_buffers.get(item.get_type_name()).unwrap();
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));

            // Set the vertex buffer for the item type.
            let index_buffer = self.index_buffers.get(item.get_type_name()).unwrap();
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // Get the bind group for the item type.
            let range = match item {
                RenderItem::Bitmap(bitmap_item) => {
                    let bind_group = self.bitmap_bind_groups.get(&bitmap_item.name).unwrap();
                    render_pass.set_bind_group(0, &bind_group.0, &[]);
                    0..6
                }
            };

            // TODO: not exactly sure what range is, looks like the # of indices - can we just do index buffer.len()?
            render_pass.draw_indexed(range, 0, 0..1);
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
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn register_bitmap(&mut self, name: &str, bitmap: NxBitmap) {
        if self.bitmap_bind_groups.contains_key(name) {
            return;
        }

        let texture_size = wgpu::Extent3d {
            width: bitmap.width.into(),
            height: bitmap.height.into(),
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // NxBitmap data is in the "reversed" bgra format.
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some(name),
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bitmap.data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * texture_size.width),
                rows_per_image: Some(texture_size.height),
            },
            texture_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
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

        self.bitmap_bind_groups
            .insert(name.to_string(), (texture_bind_group, texture));
    }
}

pub enum RenderItem {
    Bitmap(BitmapRenderItem),
}

impl RenderItem {
    pub fn get_type_name(&self) -> &str {
        match self {
            RenderItem::Bitmap(_) => "bitmap",
        }
    }
}

pub struct BitmapRenderItem {
    pub name: String,
}

impl BitmapRenderItem {
    fn create_renderer_components(renderer: &mut Renderer) {
        let render_pipeline_layout =
            renderer
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&renderer.texture_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let shader = renderer
            .device
            .create_shader_module(include_wgsl!("bitmap_shader.wgsl"));

        let render_pipeline =
            renderer
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        buffers: &[Vertex::desc()],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: renderer.config.format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                    cache: None,
                });

        const BITMAP_VERTICES: &[Vertex] = &[
            Vertex {
                position: [-0.5, -0.5, 0.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, 0.0],
                tex_coords: [0.0, 0.0],
            },
        ];

        let vertex_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("bitmap vertex buffer"),
                contents: bytemuck::cast_slice(BITMAP_VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        const BITMAP_INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

        let index_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("bitmap index buffer"),
                contents: bytemuck::cast_slice(BITMAP_INDICES),
                usage: wgpu::BufferUsages::INDEX,
            });

        renderer
            .render_pipelines
            .insert("bitmap".to_string(), render_pipeline);

        // TODO: is this right? do all bitmaps share the same vertex/index buffer, or should each bitmap get it's own?
        renderer
            .vertex_buffers
            .insert("bitmap".to_string(), vertex_buffer);

        renderer
            .index_buffers
            .insert("bitmap".to_string(), index_buffer);
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}
