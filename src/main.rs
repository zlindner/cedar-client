use std::{
    collections::HashMap,
    sync::{mpsc, Arc},
    thread,
    time::{Duration, Instant},
};

use component::Camera;
use graphics::{Renderer, RendererEvent, RendererManager};
use resource::{input::CursorState, AssetManager, Cursor, WindowProxy};
use scene::{LoginScene, Scene};
use state::State;
use system::{ui::ButtonSystem, System};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{CustomCursor, Window, WindowId},
};

mod component;
mod graphics;
mod resource;
mod scene;
mod state;
mod system;

enum WindowState {
    Uninitialized,
    Initialized(WindowManager),
}

struct WindowManager {
    sender: mpsc::Sender<WindowEvent>,
}

struct Cedar {
    window: Arc<Window>,
    state: State,
    systems: Vec<Box<dyn System>>,
    scene: Box<dyn Scene>,
    renderer_tx: mpsc::Sender<RendererEvent>,
    window_rx: mpsc::Receiver<WindowEvent>,
    custom_cursors: HashMap<CursorState, CustomCursor>,
}

impl Cedar {
    fn run(mut self) {
        self.init();

        let mut renderer_manager = RendererManager::new(self.renderer_tx.clone());

        let mut limiter = FrameLimiter::new(60);
        let mut rendered_frames = 0;
        let mut rendered_frames_tracker = Instant::now();

        loop {
            if limiter.ready_for_update() {
                self.handle_window_events();

                for system in self.systems.iter() {
                    system.execute(&mut self.state);
                }

                self.update_cursor_icon();
                limiter.last_update_start = Instant::now();
            }

            if limiter.ready_for_frame() {
                renderer_manager.generate_and_send_events(&mut self.state);

                limiter.last_frame_start = Instant::now();
                rendered_frames += 1;
            }

            if rendered_frames_tracker.elapsed() >= Duration::from_secs(1) {
                log::info!("rendered {} frames!", rendered_frames);
                rendered_frames = 0;
                rendered_frames_tracker = Instant::now();
            }

            // TODO: we should figure out the right sleep here based on frame rate.
            // Sleeping for the exact tick duration basically means it's impossible to reach our
            // target frame rate. We might need to sleep for tick duration - loop iteration duration.
            thread::sleep(limiter.tick_duration);
        }
    }

    fn init(&mut self) {
        let logical_window_size = self
            .window
            .inner_size()
            .to_logical(self.window.scale_factor());

        self.state
            .insert_resource(AssetManager::new())
            .insert_resource(Camera::new(
                logical_window_size.width,
                logical_window_size.height,
            ))
            .insert_resource(Cursor::new())
            .insert_resource(WindowProxy::new(
                self.window.inner_size(),
                self.window.scale_factor(),
            ));

        self.systems.push(Box::new(ButtonSystem::default()));

        self.scene.init(&mut self.state);
    }

    /// Handle any events sent from the ui thread.
    /// This immediately returns if no events are in the channel.
    fn handle_window_events(&self) {
        while let Ok(event) = self.window_rx.try_recv() {
            match event {
                WindowEvent::CursorMoved { position, .. } => {
                    // Since `position` is a `PhysicalPosition`, we need to apply the current scale
                    // factor to get the `LogicalPosition`.
                    let scale_factor = self.state.window().scale_factor;
                    self.state
                        .cursor()
                        .set_position(position.x / scale_factor, position.y / scale_factor);
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    self.state.cursor().add_event(button, state);
                }
                WindowEvent::Resized(new_size) => {
                    if let Err(e) = self.renderer_tx.send(RendererEvent::Resize(new_size)) {
                        log::error!("Error sending resize event to renderer: {}", e);
                    }

                    self.state
                        .window()
                        .resize(new_size, self.window.scale_factor());
                }
                _ => {}
            }
        }
    }

    fn update_cursor_icon(&self) {
        let mut cursor = self.state.cursor();

        self.window.set_cursor_visible(!cursor.should_hide());

        if !cursor.state_changed {
            return;
        }

        if let Some(custom_cursor) = self.custom_cursors.get(cursor.state()) {
            self.window.set_cursor(custom_cursor.clone());
        } else {
            log::warn!("No custom cursor found for state {:?}", cursor.state());
        }

        cursor.state_changed = false;
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

                let assets = AssetManager::new();
                let cursor = assets.get_texture("UI.nx/Basic.img/Cursor/0/0").unwrap();
                log::info!("Cursor: {:?}", cursor);

                let mut bgra = cursor.data.clone();

                for pixel in bgra.chunks_exact_mut(4) {
                    pixel.swap(0, 2); // Swap R (index 0) and B (index 2)
                }

                let mut custom_cursors = HashMap::new();
                custom_cursors.insert(
                    CursorState::Idle,
                    event_loop.create_custom_cursor(
                        CustomCursor::from_rgba(
                            bgra,
                            cursor.width as u16,
                            cursor.height as u16,
                            7,
                            7,
                        )
                        .unwrap(),
                    ),
                );

                // Create and run the main game loop.
                thread::spawn(move || {
                    let cedar = Cedar {
                        window: window.clone(),
                        state: State::new(),
                        systems: Vec::new(),
                        scene: Box::new(LoginScene::default()),
                        renderer_tx,
                        window_rx: window_rx,
                        custom_cursors,
                    };

                    cedar.run();
                });

                let manager = WindowManager { sender: window_tx };
                *self = WindowState::Initialized(manager);
            }
            WindowState::Initialized(_) => return,
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let manager = match self {
            WindowState::Uninitialized => return,
            WindowState::Initialized(manager) => manager,
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            e => {
                if let Err(e) = manager.sender.send(e) {
                    log::error!("Error sending window event: {}", e);
                }
            }
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
    target_update_duration: Duration,
    last_update_start: Instant,
    target_frame_duration: Duration,
    last_frame_start: Instant,
}

impl FrameLimiter {
    pub fn new(target_fps: u32) -> Self {
        Self {
            tick_duration: Duration::from_secs(1) / 120,
            target_update_duration: Duration::from_secs(1) / target_fps,
            last_update_start: Instant::now(),
            target_frame_duration: Duration::from_secs(1) / target_fps,
            last_frame_start: Instant::now(),
        }
    }

    pub fn ready_for_update(&self) -> bool {
        Instant::now() - self.last_update_start > self.target_update_duration
    }

    pub fn ready_for_frame(&self) -> bool {
        Instant::now() - self.last_frame_start > self.target_frame_duration
    }
}
