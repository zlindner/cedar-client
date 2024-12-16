use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
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
    fn run() {
        let event_loop = EventLoop::new().expect("event loop should be created");
        event_loop.set_control_flow(ControlFlow::Poll);

        event_loop
            .run_app(&mut AppState::Uninitialized)
            .expect("event loop should run");
    }

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
}

impl ApplicationHandler for AppState {
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // TODO:
            }
            _ => (),
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match self {
            AppState::Uninitialized => *self = AppState::Initialized(App::new(event_loop)),
            _ => return,
        }
    }
}

struct Gpu {
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
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

        Self {
            instance,
            surface,
            adapter,
            device,
            queue,
        }
    }
}

fn main() {
    // TODO: setup tracing

    App::run();
}
