use std::{
    collections::HashMap,
    sync::{mpsc, Arc},
    thread,
    time::{Duration, Instant},
};

use component::Camera;
use graphics::{RenderQueueBuilder, Renderer, RendererEvent};
use resource::{input::CursorState, AssetManager, Cursor, GameKey, Keyboard, WindowProxy};
use scene::{GameScene, LoginScene, Scene};
use state::State;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize},
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    keyboard::{Key, NamedKey},
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
const START_IN_GAME_SCENE: bool = true;
const FIXED_UPDATE_DURATION: Duration = Duration::from_millis(8);
const MAX_FIXED_UPDATE_ACCUMULATION: Duration = Duration::from_millis(250);
const TARGET_FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / 60);

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
    TextInput {
        text: String,
    },
    Backspace,
    Delete,
    Left,
    Right,
    Home,
    End,
    Tab,
    Enter,
    KeyInput {
        key: GameKey,
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
    fixed_update_systems: Vec<fn(&mut State)>,
    input_systems: Vec<fn(&mut State)>,
    end_of_loop_systems: Vec<fn(&mut State)>,
    scene: Box<dyn Scene>,
    renderer_tx: EventLoopProxy<RendererEvent>,
    window_rx: mpsc::Receiver<GameWindowEvent>,
    custom_cursors: HashMap<CursorState, CustomCursor>,
}

impl Cedar {
    fn run(mut self) {
        self.init();

        let mut render_queue_builder = RenderQueueBuilder::new(self.renderer_tx.clone());

        let mut clock = GameClock::new();
        loop {
            let now = Instant::now();
            clock.record_elapsed(now);

            self.handle_window_events();

            for system in self.input_systems.iter() {
                (system)(&mut self.state);
            }

            while clock.ready_for_fixed_update() {
                for system in self.fixed_update_systems.iter() {
                    (system)(&mut self.state);
                }

                clock.mark_fixed_update_finished();
            }

            if clock.ready_for_frame(now) {
                render_queue_builder.generate_and_send_events(&mut self.state);

                clock.mark_frame_finished(now);
            }

            for system in self.end_of_loop_systems.iter() {
                (system)(&mut self.state);
            }

            self.update_cursor_icon();

            match clock.sleep_duration(Instant::now()) {
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
            .insert_resource(Keyboard::new())
            .insert_resource(WindowProxy::new(
                self.initial_window_size,
                self.initial_scale_factor,
            ));

        self.fixed_update_systems
            .push(system::player::player_movement_system);
        self.input_systems.push(system::ui::button_system);
        self.input_systems.push(system::ui::text_input_system);
        self.input_systems.push(system::ui::text_system);
        self.end_of_loop_systems
            .push(system::ui::clear_input_events_system);

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
                GameWindowEvent::TextInput { text } => {
                    self.state.keyboard().add_text(&text);
                }
                GameWindowEvent::Backspace => {
                    self.state.keyboard().add_backspace();
                }
                GameWindowEvent::Delete => {
                    self.state.keyboard().add_delete();
                }
                GameWindowEvent::Left => {
                    self.state.keyboard().add_left();
                }
                GameWindowEvent::Right => {
                    self.state.keyboard().add_right();
                }
                GameWindowEvent::Home => {
                    self.state.keyboard().add_home();
                }
                GameWindowEvent::End => {
                    self.state.keyboard().add_end();
                }
                GameWindowEvent::Tab => {
                    self.state.keyboard().add_tab();
                }
                GameWindowEvent::Enter => {
                    self.state.keyboard().add_enter();
                }
                GameWindowEvent::KeyInput { key, state } => {
                    self.state.keyboard().set_key_state(key, state);
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
                        fixed_update_systems: Vec::new(),
                        input_systems: Vec::new(),
                        end_of_loop_systems: Vec::new(),
                        scene: initial_scene(),
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
            WindowEvent::KeyboardInput { event, .. } => {
                let movement_key = match event.logical_key {
                    Key::Named(NamedKey::ArrowLeft) => Some(GameKey::Left),
                    Key::Named(NamedKey::ArrowRight) => Some(GameKey::Right),
                    Key::Named(NamedKey::ArrowUp) => Some(GameKey::Up),
                    Key::Named(NamedKey::ArrowDown) => Some(GameKey::Down),
                    Key::Named(NamedKey::Space) => Some(GameKey::Jump),
                    _ => None,
                };

                if let Some(key) = movement_key {
                    if let Err(e) = manager.sender.send(GameWindowEvent::KeyInput {
                        key,
                        state: event.state,
                    }) {
                        log::error!("Error sending window event: {}", e);
                    }
                }

                if event.state != ElementState::Pressed {
                    return;
                }

                match event.logical_key {
                    Key::Named(NamedKey::Backspace) => {
                        if let Err(e) = manager.sender.send(GameWindowEvent::Backspace) {
                            log::error!("Error sending window event: {}", e);
                        }
                    }
                    Key::Named(NamedKey::Delete) => {
                        if let Err(e) = manager.sender.send(GameWindowEvent::Delete) {
                            log::error!("Error sending window event: {}", e);
                        }
                    }
                    Key::Named(NamedKey::ArrowLeft) => {
                        if let Err(e) = manager.sender.send(GameWindowEvent::Left) {
                            log::error!("Error sending window event: {}", e);
                        }
                    }
                    Key::Named(NamedKey::ArrowRight) => {
                        if let Err(e) = manager.sender.send(GameWindowEvent::Right) {
                            log::error!("Error sending window event: {}", e);
                        }
                    }
                    Key::Named(NamedKey::Home) => {
                        if let Err(e) = manager.sender.send(GameWindowEvent::Home) {
                            log::error!("Error sending window event: {}", e);
                        }
                    }
                    Key::Named(NamedKey::End) => {
                        if let Err(e) = manager.sender.send(GameWindowEvent::End) {
                            log::error!("Error sending window event: {}", e);
                        }
                    }
                    Key::Named(NamedKey::Tab) => {
                        if let Err(e) = manager.sender.send(GameWindowEvent::Tab) {
                            log::error!("Error sending window event: {}", e);
                        }
                    }
                    Key::Named(NamedKey::Enter) => {
                        if let Err(e) = manager.sender.send(GameWindowEvent::Enter) {
                            log::error!("Error sending window event: {}", e);
                        }
                    }
                    _ => {
                        if let Some(text) = event.text {
                            if let Err(e) = manager
                                .sender
                                .send(GameWindowEvent::TextInput { text: text.into() })
                            {
                                log::error!("Error sending window event: {}", e);
                            }
                        }
                    }
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

fn initial_scene() -> Box<dyn Scene> {
    if START_IN_GAME_SCENE {
        Box::new(GameScene)
    } else {
        Box::new(LoginScene)
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

struct GameClock {
    previous_tick_at: Instant,
    fixed_update_accumulator: Duration,
    next_frame_at: Instant,
}

impl GameClock {
    pub fn new() -> Self {
        let now = Instant::now();

        Self {
            previous_tick_at: now,
            fixed_update_accumulator: Duration::ZERO,
            next_frame_at: now + TARGET_FRAME_DURATION,
        }
    }

    pub fn record_elapsed(&mut self, now: Instant) {
        let elapsed = now.saturating_duration_since(self.previous_tick_at);
        self.previous_tick_at = now;
        self.fixed_update_accumulator =
            (self.fixed_update_accumulator + elapsed).min(MAX_FIXED_UPDATE_ACCUMULATION);
    }

    pub fn ready_for_fixed_update(&self) -> bool {
        self.fixed_update_accumulator >= FIXED_UPDATE_DURATION
    }

    pub fn mark_fixed_update_finished(&mut self) {
        self.fixed_update_accumulator -= FIXED_UPDATE_DURATION;
    }

    pub fn ready_for_frame(&self, now: Instant) -> bool {
        now >= self.next_frame_at
    }

    pub fn mark_frame_finished(&mut self, now: Instant) {
        self.next_frame_at = next_deadline(self.next_frame_at, TARGET_FRAME_DURATION, now);
    }

    pub fn sleep_duration(&self, now: Instant) -> Option<Duration> {
        let next_fixed_update_at = if self.ready_for_fixed_update() {
            now
        } else {
            now + (FIXED_UPDATE_DURATION - self.fixed_update_accumulator)
        };
        let next_deadline = next_fixed_update_at.min(self.next_frame_at);
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
