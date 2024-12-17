use std::sync::Arc;

use gpu::Gpu;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

mod gpu;
mod texture;

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

fn main() {
    // TODO: setup tracing

    App::run();
}
