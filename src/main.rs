use std::{
    path::Path,
    sync::{mpsc, Arc},
    thread,
    time::{Duration, Instant},
};

use ecs::World;
use graphics::{BitmapRenderItem, RenderItem, Renderer, RendererEvent};
use resource::{AssetManager, WindowProxy};
use scene::{LoginScene, Scene};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent as WinitWindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Cursor, CustomCursor, Window, WindowId},
};

mod ecs;
mod graphics;
mod resource;
mod scene;

enum WindowState {
    Uninitialized,
    Initialized(WindowEventHandler),
}

struct WindowEventHandler {
    sender: mpsc::Sender<WindowEvent>,
}

struct WindowEvent(WinitWindowEvent);

struct Cedar {
    window: Arc<Window>,
    world: World,
    scene: Box<dyn Scene>,
    renderer_tx: mpsc::Sender<RendererEvent>,
    window_rx: mpsc::Receiver<WindowEvent>,
}

impl Cedar {
    fn new(event_loop: &ActiveEventLoop) {
        /*let ui_nx = NxFile::open(Path::new("nx/UI.nx")).unwrap();
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
        );*/
    }

    fn run(mut self) {
        self.init();

        let mut limiter = FrameLimiter::new(60);
        let mut rendered_frames = 0;
        let mut rendered_frames_tracker = Instant::now();

        loop {
            if limiter.ready_for_next_frame() {
                let assets = self.world.assets();
                let mut items = Vec::new();

                // TODO: this renders all registered bitmaps, we probably need some "should_render/hidden" flag.
                for bitmap in assets.get_bitmaps().iter() {
                    items.push(RenderItem::Bitmap(BitmapRenderItem {
                        name: bitmap.to_string(),
                    }));
                }

                if let Err(e) = self.renderer_tx.send(RendererEvent::Render(items)) {
                    log::error!("Error sending render event: {}", e);
                }

                limiter.last_frame_start = Instant::now();
                rendered_frames += 1;
            }

            if rendered_frames_tracker.elapsed() >= Duration::from_secs(1) {
                log::info!("rendered {} frames!", rendered_frames);
                rendered_frames = 0;
                rendered_frames_tracker = Instant::now();
            }

            // TODO: we should figure out the right sleep here based on frame rate.
            // Not sleeping causes 100% cpu usage.
            // thread::sleep(limiter.tick_duration);
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

impl ApplicationHandler for WindowState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match self {
            WindowState::Uninitialized => {
                let window_attributes = Window::default_attributes()
                    .with_title("CedarMS")
                    .with_inner_size(LogicalSize::new(800, 600));

                let window = Arc::new(
                    event_loop
                        .create_window(window_attributes)
                        .expect("window should be created"),
                );

                // Initialize the renderer passing it the event receiver.
                // The channel is used for other components to send updates directly to the renderer,
                // ex. an entity was added to the world to be rendered, an asset was registered, etc.
                let (renderer_tx, renderer_rx) = mpsc::channel::<RendererEvent>();
                let renderer =
                    futures::executor::block_on(Renderer::new(window.clone(), renderer_rx));

                // Start a new thread for the renderer.
                // NOTE: creating the renderer must be done on the main thread.
                thread::spawn(move || renderer.run());

                let (window_tx, window_rx) = mpsc::channel::<WindowEvent>();

                // Create and run the main game loop.
                thread::spawn(move || {
                    // TODO: maybe Cedar::run() makes more sense.
                    let cedar = Cedar {
                        window: window.clone(),
                        world: World::new(),
                        scene: Box::new(LoginScene::default()),
                        renderer_tx,
                        window_rx: window_rx,
                    };

                    cedar.run();
                });

                let handler = WindowEventHandler { sender: window_tx };
                *self = WindowState::Initialized(handler);
            }
            WindowState::Initialized(_) => return,
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WinitWindowEvent,
    ) {
        let handler = match self {
            WindowState::Uninitialized => return,
            WindowState::Initialized(handler) => handler,
        };

        match event {
            WinitWindowEvent::CloseRequested => {
                event_loop.exit();
            }
            e => {
                // I'm pretty sure we can't send the WinitWindowEvent directly, should confirm this.
                if let Err(e) = handler.sender.send(WindowEvent(e)) {
                    log::error!("Error sending window event: {}", e);
                }
            } /*
              WindowEvent::Resized(new_size) => {
                  cedar
                      .renderer_tx
                      .send(RendererEvent::Resize(new_size))
                      .unwrap();

                  cedar
                      .world
                      .window()
                      .resize(new_size, cedar.window.scale_factor());
              }*/
        }
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let event_loop = EventLoop::new().expect("event loop should be created");
    event_loop.set_control_flow(ControlFlow::Wait);

    event_loop
        .run_app(&mut WindowState::Uninitialized)
        .expect("event loop should run");
}

struct FrameLimiter {
    tick_duration: Duration,
    target_frame_duration: Duration,
    last_frame_start: Instant,
}

impl FrameLimiter {
    pub fn new(target_fps: u32) -> Self {
        Self {
            tick_duration: Duration::from_secs(1) / 120,
            target_frame_duration: Duration::from_secs(1) / target_fps,
            last_frame_start: Instant::now(),
        }
    }

    pub fn ready_for_next_frame(&self) -> bool {
        Instant::now() - self.last_frame_start > self.target_frame_duration
    }
}
