use std::{
    collections::HashMap,
    sync::{mpsc, Arc},
    thread,
    time::{Duration, Instant},
};

use component::Camera;
use graphics::{RenderQueueBuilder, Renderer, RendererEvent};
use resource::{input::CursorState, AssetManager, Cursor, WindowProxy};
use scene::{LoginScene, Scene};
use state::State;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize},
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    window::{CustomCursor, Window, WindowId},
};

mod component;
mod graphics;
mod resource;
mod scene;
mod state;
mod system;

const VIRTUAL_WIDTH: f32 = 800.0;
const VIRTUAL_HEIGHT: f32 = 600.0;

enum WindowState {
    Uninitialized(EventLoopProxy<RendererEvent>),
    Initialized(WindowManager),
}

struct WindowManager {
    window: Arc<Window>,
    renderer: Renderer,
    sender: mpsc::Sender<GameWindowEvent>,
    logical_width: f32,
    logical_height: f32,
}

enum GameWindowEvent {
    CursorMoved {
        x: f64,
        y: f64,
    },
    MouseInput {
        button: MouseButton,
        state: ElementState,
    },
    Resized {
        physical_size: winit::dpi::PhysicalSize<u32>,
        scale_factor: f64,
    },
}

struct Cedar {
    window: Arc<Window>,
    initial_window_size: winit::dpi::PhysicalSize<u32>,
    initial_scale_factor: f64,
    asset_manager: Option<AssetManager>,
    state: State,
    systems: Vec<fn(&mut State)>,
    scene: Box<dyn Scene>,
    renderer_tx: EventLoopProxy<RendererEvent>,
    window_rx: mpsc::Receiver<GameWindowEvent>,
    custom_cursors: HashMap<CursorState, CustomCursor>,
}

impl Cedar {
    fn run(mut self) {
        self.init();

        let mut render_queue_builder = RenderQueueBuilder::new(self.renderer_tx.clone());

        let mut limiter = FrameLimiter::new(60);
        loop {
            let now = Instant::now();

            if limiter.ready_for_update(now) {
                self.handle_window_events();

                for system in self.systems.iter() {
                    (system)(&mut self.state);
                }

                self.update_cursor_icon();
                limiter.mark_update_finished(Instant::now());
            }

            let now = Instant::now();

            if limiter.ready_for_frame(now) {
                render_queue_builder.generate_and_send_events(&mut self.state);

                limiter.mark_frame_finished(Instant::now());
            }

            match limiter.sleep_duration(Instant::now()) {
                Some(duration) => thread::sleep(duration),
                None => thread::yield_now(),
            }
        }
    }

    fn init(&mut self) {
        let logical_window_size = self
            .initial_window_size
            .to_logical(self.initial_scale_factor);

        self.state
            .insert_resource(Camera::new(
                logical_window_size.width,
                logical_window_size.height,
            ))
            .insert_resource(
                self.asset_manager
                    .take()
                    .expect("AssetManager should only be inserted once"),
            )
            .insert_resource(Cursor::new())
            .insert_resource(WindowProxy::new(
                self.initial_window_size,
                self.initial_scale_factor,
            ));

        self.systems.push(system::ui::button_system);
        self.systems.push(system::ui::text_system);

        self.scene.init(&mut self.state);
    }

    /// Handle any events sent from the ui thread.
    /// This immediately returns if no events are in the channel.
    fn handle_window_events(&self) {
        while let Ok(event) = self.window_rx.try_recv() {
            match event {
                GameWindowEvent::CursorMoved { x, y } => {
                    self.state.cursor().set_position(x, y);
                }
                GameWindowEvent::MouseInput { button, state } => {
                    self.state.cursor().add_event(button, state);
                }
                GameWindowEvent::Resized {
                    physical_size,
                    scale_factor,
                } => {
                    self.state.window().resize(physical_size, scale_factor);
                }
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

impl ApplicationHandler<RendererEvent> for WindowState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match self {
            WindowState::Uninitialized(renderer_tx) => {
                let window_attributes = Window::default_attributes()
                    .with_title("CedarMS")
                    .with_inner_size(LogicalSize::new(800, 600));

                let window = Arc::new(
                    event_loop
                        .create_window(window_attributes)
                        .expect("window should be created"),
                );

                // Initialize the renderer on the main thread so surface configure/present calls
                // stay on the same thread as the winit window.
                let renderer = futures::executor::block_on(Renderer::new(window.clone()));
                let renderer_tx = renderer_tx.clone();

                let initial_window_size = window.inner_size();
                let initial_scale_factor = window.scale_factor();
                let initial_logical_size = initial_window_size.to_logical(initial_scale_factor);

                let (window_tx, window_rx) = mpsc::channel::<GameWindowEvent>();

                let mut asset_manager = AssetManager::new();
                let cursor = asset_manager
                    .get_texture_rgba("UI.nx/Basic.img/Cursor/0/0")
                    .unwrap();

                let mut custom_cursors = HashMap::new();
                custom_cursors.insert(
                    CursorState::Idle,
                    event_loop.create_custom_cursor(
                        CustomCursor::from_rgba(
                            cursor.image.data.clone(),
                            cursor.image.width as u16,
                            cursor.image.height as u16,
                            7,
                            7,
                        )
                        .unwrap(),
                    ),
                );

                // Create and run the main game loop.
                let game_window = window.clone();
                thread::spawn(move || {
                    let cedar = Cedar {
                        window: game_window,
                        initial_window_size,
                        initial_scale_factor,
                        asset_manager: Some(asset_manager),
                        state: State::new(),
                        systems: Vec::new(),
                        scene: Box::new(LoginScene::default()),
                        renderer_tx,
                        window_rx,
                        custom_cursors,
                    };

                    cedar.run();
                });

                let manager = WindowManager {
                    window: window.clone(),
                    renderer,
                    sender: window_tx,
                    logical_width: initial_logical_size.width,
                    logical_height: initial_logical_size.height,
                };
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
            WindowState::Uninitialized(_) => return,
            WindowState::Initialized(manager) => manager,
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let logical_position: LogicalPosition<f64> =
                    position.to_logical(manager.window.scale_factor());
                let virtual_x =
                    logical_position.x * f64::from(VIRTUAL_WIDTH / manager.logical_width);
                let virtual_y =
                    logical_position.y * f64::from(VIRTUAL_HEIGHT / manager.logical_height);

                if let Err(e) = manager.sender.send(GameWindowEvent::CursorMoved {
                    x: virtual_x,
                    y: virtual_y,
                }) {
                    log::error!("Error sending window event: {}", e);
                }
            }
            WindowEvent::MouseInput { button, state, .. } => {
                if let Err(e) = manager
                    .sender
                    .send(GameWindowEvent::MouseInput { button, state })
                {
                    log::error!("Error sending window event: {}", e);
                }
            }
            WindowEvent::Resized(physical_size) => {
                let scale_factor = manager.window.scale_factor();
                let logical_size = physical_size.to_logical(scale_factor);
                manager.logical_width = logical_size.width;
                manager.logical_height = logical_size.height;
                manager.renderer.resize(physical_size);

                if let Err(e) = manager.sender.send(GameWindowEvent::Resized {
                    physical_size,
                    scale_factor,
                }) {
                    log::error!("Error sending window event: {}", e);
                }
            }
            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: RendererEvent) {
        let manager = match self {
            WindowState::Uninitialized(_) => return,
            WindowState::Initialized(manager) => manager,
        };

        manager.renderer.handle_event(event);
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let event_loop = EventLoop::<RendererEvent>::with_user_event()
        .build()
        .expect("event loop should be created");
    let renderer_tx = event_loop.create_proxy();
    event_loop.set_control_flow(ControlFlow::Wait);

    event_loop
        .run_app(&mut WindowState::Uninitialized(renderer_tx))
        .expect("event loop should run");
}

struct FrameLimiter {
    target_update_duration: Duration,
    next_update_at: Instant,
    target_frame_duration: Duration,
    next_frame_at: Instant,
}

impl FrameLimiter {
    pub fn new(target_fps: u32) -> Self {
        let now = Instant::now();
        let target_duration = Duration::from_secs(1) / target_fps;

        Self {
            target_update_duration: target_duration,
            next_update_at: now + target_duration,
            target_frame_duration: target_duration,
            next_frame_at: now + target_duration,
        }
    }

    pub fn ready_for_update(&self, now: Instant) -> bool {
        now >= self.next_update_at
    }

    pub fn ready_for_frame(&self, now: Instant) -> bool {
        now >= self.next_frame_at
    }

    pub fn mark_update_finished(&mut self, now: Instant) {
        self.next_update_at = next_deadline(self.next_update_at, self.target_update_duration, now);
    }

    pub fn mark_frame_finished(&mut self, now: Instant) {
        self.next_frame_at = next_deadline(self.next_frame_at, self.target_frame_duration, now);
    }

    pub fn sleep_duration(&self, now: Instant) -> Option<Duration> {
        let next_deadline = self.next_update_at.min(self.next_frame_at);
        next_deadline.checked_duration_since(now)
    }
}

fn next_deadline(mut deadline: Instant, target_duration: Duration, now: Instant) -> Instant {
    deadline += target_duration;

    if deadline <= now {
        now + target_duration
    } else {
        deadline
    }
}
