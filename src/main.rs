use std::sync::Arc;

use wgpu::include_wgsl;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

enum AppState {
    Uninitialized,
    Initialized(App),
}

struct App {
    window: Arc<Window>,
    gpu: Gpu,
}

impl App {
    fn new(event_loop: &ActiveEventLoop) -> Self {
        let window_attributes = Window::default_attributes().with_title("CedarMS");

        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("window should be created"),
        );

        // TODO: we might eventually want an actual runtime for connection handling.
        // I think all of the winit + wgpu stuff needs to be created on the main thread.
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("tokio runtime should be created");

        let gpu = runtime.block_on(Gpu::new(window.clone()));

        Self { window, gpu }
    }

    fn run() {
        let event_loop = EventLoop::new().expect("event loop should be created");
        event_loop.set_control_flow(ControlFlow::Poll);

        event_loop
            .run_app(&mut AppState::Uninitialized)
            .expect("event loop should run");
    }

    fn render(&mut self, event_loop: &ActiveEventLoop) {
        match self.gpu.render() {
            Ok(_) => {}
            // Reconfigure the surface if it's lost/outdated.
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.resize(self.window.inner_size());
            }
            // The system is OOM - we should quit the app.
            Err(wgpu::SurfaceError::OutOfMemory) => {
                // TODO: log an error.
                event_loop.exit();
            }
            // A frame took too long to render.
            Err(wgpu::SurfaceError::Timeout) => {
                // TODO: log a warning.
            }
        };
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.gpu.resize(new_size);
        }
    }
}

impl ApplicationHandler for AppState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match self {
            AppState::Uninitialized => *self = AppState::Initialized(App::new(event_loop)),
            AppState::Initialized(_) => return,
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let app = match self {
            AppState::Uninitialized => return,
            AppState::Initialized(app) => app,
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                app.window.request_redraw();

                // TODO: we should call some `app.update()` fn here.

                app.render(event_loop);
            }
            WindowEvent::Resized(new_size) => {
                app.resize(new_size);
            }
            WindowEvent::CursorMoved {
                position: new_position,
                ..
            } => {
                // TODO: we should call some `cursor.update(new_position)` here.
            }
            _ => (),
        }
    }
}

struct Gpu {
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
}

impl Gpu {
    async fn new(window: Arc<Window>) -> Self {
        // TODO: do we need to keep this around?
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

        let size = window.inner_size();

        // TODO: we can modify this to support things such as vsync, different frame rates, etc.
        let surface_config = surface
            .get_default_config(&adapter, size.width, size.height)
            .expect("surface configuration should be created");

        surface.configure(&device, &surface_config);

        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE.
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL.
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION.
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

        Self {
            instance,
            surface,
            adapter,
            device,
            queue,
            surface_config,
            render_pipeline,
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
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

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.draw(0..3, 0..1);

        // The render pass must be dropped before calling `encoder.finish()`.
        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.surface_config.width = new_size.width;
        self.surface_config.height = new_size.height;
        self.surface.configure(&self.device, &self.surface_config);
    }
}

fn main() {
    // TODO: setup tracing

    App::run();
}
