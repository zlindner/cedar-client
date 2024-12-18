use std::{
    path::Path,
    sync::{mpsc, Arc},
};

use ecs::World;
use graphics::{BitmapRenderItem, RenderItem, Renderer, RendererEvent};
use nx_pkg4::{file::NxFile, node::Node};
use resource::{AssetManager, WindowProxy};
use scene::{LoginScene, Scene};
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
mod scene;

enum CedarState {
    Uninitialized,
    Initialized(Cedar),
}

struct Cedar {
    window: Arc<Window>,
    renderer_tx: mpsc::Sender<RendererEvent>,
    world: World,
    scene: Box<dyn Scene>,
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

        // Initialize the renderer, passing it the channel receiver.
        // The channel is used for other components to send updates directly to the renderer,
        // ex. an entity was added to the world to be rendered, an asset was registered, etc.
        let (renderer_tx, renderer_rx) = mpsc::channel::<RendererEvent>();
        let renderer = futures::executor::block_on(Renderer::new(window.clone(), renderer_rx));

        // Start a new thread for the renderer.
        // NOTE: creating the renderer must be done on the main thread.
        std::thread::spawn(move || renderer.run());

        Self {
            window,
            renderer_tx,
            world: World::new(),
            scene: Box::new(LoginScene::default()),
        }
    }

    fn init(&mut self) {
        // Add default resources to the world.
        self.world
            .insert_resource(AssetManager::new(self.renderer_tx.clone()));
        self.world.insert_resource(WindowProxy::new(
            self.window.inner_size(),
            self.window.scale_factor(),
        ));

        self.scene.init(&mut self.world);
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
            // FIXME: this sends way too many updates and floods the channel.
            // Need to keep track of time here.
            WindowEvent::RedrawRequested => {
                cedar.window.request_redraw();

                let assets = cedar.world.assets();
                let mut items = Vec::new();

                // TODO: this renders all registered bitmaps, we probably need some "should_render/hidden" flag.
                for bitmap in assets.get_bitmaps().iter() {
                    items.push(RenderItem::Bitmap(BitmapRenderItem {
                        name: bitmap.to_string(),
                    }));
                }

                cedar
                    .renderer_tx
                    .send(RendererEvent::Render(items))
                    .unwrap();
            }
            WindowEvent::Resized(new_size) => {
                cedar
                    .renderer_tx
                    .send(RendererEvent::Resize(new_size))
                    .unwrap();

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
