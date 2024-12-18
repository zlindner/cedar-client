use std::{path::Path, sync::Arc};

use ecs::World;
use graphics::{BitmapRenderItem, RenderItem, Renderer};
use nx_pkg4::{file::NxFile, node::Node};
use resource::{asset_manager::NxFileType, AssetManager, WindowProxy};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Cursor, CustomCursor, Window, WindowId},
};

mod ecs;
mod graphics;
mod resource;

enum CedarState {
    Uninitialized,
    Initialized(Cedar),
}

struct Cedar {
    window: Arc<Window>,
    renderer: Renderer,
    world: World,
}

impl Cedar {
    fn new(event_loop: &ActiveEventLoop) -> Self {
        let ui_nx = NxFile::open(Path::new("nx/UI.nx")).unwrap();
        let root = ui_nx.root();
        let nx_cursor = root
            .get("Basic.img")
            .get("Cursor")
            .get("0")
            .get("0")
            .bitmap()
            .unwrap()
            .unwrap();

        let mut bgra = nx_cursor.data.clone();

        for pixel in bgra.chunks_exact_mut(4) {
            pixel.swap(0, 2); // Swap R (index 0) and B (index 2)
        }

        // TODO: we need to call window.set_cursor when required to change cursor icon.
        // TODO: we need to figure out the right x and y hotspots.
        let cursor = event_loop.create_custom_cursor(
            CustomCursor::from_rgba(bgra, nx_cursor.width, nx_cursor.height, 7, 7).unwrap(),
        );

        let window_attributes = Window::default_attributes()
            .with_title("CedarMS")
            .with_inner_size(LogicalSize::new(800, 600))
            .with_cursor(Cursor::Custom(cursor));

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

        let renderer = runtime.block_on(Renderer::new(window.clone()));

        Self {
            window,
            renderer,
            world: World::new(),
        }
    }

    fn init(&mut self) {
        self.renderer.init();

        self.world.insert_resource(AssetManager::new());
        self.world.insert_resource(WindowProxy::new(
            self.window.inner_size(),
            self.window.scale_factor(),
        ));
    }

    fn render(&mut self, event_loop: &ActiveEventLoop) {
        let mut items = Vec::new();

        let assets = self.world.assets();

        let nexon = assets
            .nx(NxFileType::Map001)
            .get("Back")
            .get("login.img")
            .get("back")
            .get("11")
            .bitmap()
            .unwrap()
            .unwrap();

        // HACK: find a place to register the bitmap with the renderer, probably somewhere in world/assetmanager.
        self.renderer
            .register_bitmap("Back/login.img/back/11", nexon);

        items.push(RenderItem::Bitmap(BitmapRenderItem {
            name: "Back/login.img/back/11".to_string(),
        }));

        // TODO: populate items with all of the render items.
        // ex. for _ in world.query<&Bitmap>().iter()...

        match self.renderer.render(items) {
            Ok(_) => {}
            // Reconfigure the surface if it's lost/outdated.
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.renderer.resize(self.window.inner_size());
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
}

impl ApplicationHandler for CedarState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match self {
            CedarState::Uninitialized => {
                let mut cedar = Cedar::new(event_loop);
                cedar.init();
                *self = CedarState::Initialized(cedar);
            }
            CedarState::Initialized(_) => return,
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let cedar = match self {
            CedarState::Uninitialized => return,
            CedarState::Initialized(app) => app,
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                cedar.window.request_redraw();

                // TODO: we should call some `app.update()` fn here.

                cedar.render(event_loop);
            }
            WindowEvent::Resized(new_size) => {
                cedar.renderer.resize(new_size);
                cedar
                    .world
                    .window()
                    .resize(new_size, cedar.window.scale_factor());
            }
            _ => (),
        }
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let event_loop = EventLoop::new().expect("event loop should be created");
    event_loop.set_control_flow(ControlFlow::Poll);

    event_loop
        .run_app(&mut CedarState::Uninitialized)
        .expect("event loop should run");
}
