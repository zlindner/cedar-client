use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use winit::event::{ElementState, MouseButton};

// TODO: figure out what the right value for this should be.
/// The number of seconds after which we will hide the cursor if it hasn't moved.
const HIDE_AFTER_SECONDS: u64 = 5;

#[derive(Debug)]
pub struct Cursor {
    x: f64,
    y: f64,
    state: CursorState,

    /// Whether the cursor's state recently changed.
    pub state_changed: bool,

    pressed_buttons: HashSet<MouseButton>,
    events: Vec<(MouseButton, ElementState)>,

    /// The instant when the cursor was last moved.
    last_moved: Instant,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            state: CursorState::Idle,
            // This should be initially set to true to ensure we set the default idle icon.
            state_changed: true,
            pressed_buttons: HashSet::new(),
            events: Vec::new(),
            last_moved: Instant::now(),
        }
    }

    pub fn position(&self) -> (f64, f64) {
        (self.x, self.y)
    }

    pub fn set_position(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;

        // Update the time at which the cursor was last moved.
        // This is used to hide the cursor after not moving for a while.
        self.last_moved = Instant::now();
    }

    pub fn state(&self) -> &CursorState {
        &self.state
    }

    pub fn set_state(&mut self, state: CursorState) {
        if self.state != state {
            self.state_changed = true;
        }

        self.state = state;
    }

    pub fn add_event(&mut self, button: MouseButton, state: ElementState) {
        match state {
            ElementState::Pressed => self.pressed_buttons.insert(button),
            ElementState::Released => self.pressed_buttons.remove(&button),
        };

        self.events.push((button, state));
    }

    /// Whether the cursor should be hidden.
    pub fn should_hide(&self) -> bool {
        // TODO: there are certain states where we should always return false, ex. grabbing.
        Instant::now() - self.last_moved > Duration::from_secs(HIDE_AFTER_SECONDS)
    }

    pub fn is_button_pressed(&self, button: MouseButton) -> bool {
        self.pressed_buttons.contains(&button)
    }

    pub fn was_button_pressed(&self, button: MouseButton) -> bool {
        self.events
            .iter()
            .any(|(event_button, state)| *event_button == button && *state == ElementState::Pressed)
    }

    pub fn clear_events(&mut self) {
        self.events.clear();
    }
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum CursorState {
    Idle,
    Hidden,
}

pub struct Keyboard {
    held_keys: HashSet<GameKey>,
    pressed_keys: HashSet<GameKey>,
    text: String,
    backspaces: usize,
    deletes: usize,
    left_pressed: bool,
    right_pressed: bool,
    home_pressed: bool,
    end_pressed: bool,
    tab_pressed: bool,
    enter_pressed: bool,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            held_keys: HashSet::new(),
            pressed_keys: HashSet::new(),
            text: String::new(),
            backspaces: 0,
            deletes: 0,
            left_pressed: false,
            right_pressed: false,
            home_pressed: false,
            end_pressed: false,
            tab_pressed: false,
            enter_pressed: false,
        }
    }

    pub fn add_text(&mut self, text: &str) {
        self.text.extend(
            text.chars()
                .filter(|character| !character.is_control() && *character != '\u{7f}'),
        );
    }

    pub fn set_key_state(&mut self, key: GameKey, state: ElementState) {
        match state {
            ElementState::Pressed => {
                self.held_keys.insert(key);
                self.pressed_keys.insert(key);
            }
            ElementState::Released => {
                self.held_keys.remove(&key);
            }
        };
    }

    pub fn add_backspace(&mut self) {
        self.backspaces += 1;
    }

    pub fn add_delete(&mut self) {
        self.deletes += 1;
    }

    pub fn add_left(&mut self) {
        self.left_pressed = true;
    }

    pub fn add_right(&mut self) {
        self.right_pressed = true;
    }

    pub fn add_home(&mut self) {
        self.home_pressed = true;
    }

    pub fn add_end(&mut self) {
        self.end_pressed = true;
    }

    pub fn add_tab(&mut self) {
        self.tab_pressed = true;
    }

    pub fn add_enter(&mut self) {
        self.enter_pressed = true;
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn backspaces(&self) -> usize {
        self.backspaces
    }

    pub fn deletes(&self) -> usize {
        self.deletes
    }

    pub fn left_pressed(&self) -> bool {
        self.left_pressed
    }

    pub fn right_pressed(&self) -> bool {
        self.right_pressed
    }

    pub fn home_pressed(&self) -> bool {
        self.home_pressed
    }

    pub fn end_pressed(&self) -> bool {
        self.end_pressed
    }

    pub fn tab_pressed(&self) -> bool {
        self.tab_pressed
    }

    pub fn enter_pressed(&self) -> bool {
        self.enter_pressed
    }

    pub fn key_down(&self, key: GameKey) -> bool {
        self.held_keys.contains(&key)
    }

    pub fn key_pressed(&self, key: GameKey) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn clear_events(&mut self) {
        self.pressed_keys.clear();
        self.text.clear();
        self.backspaces = 0;
        self.deletes = 0;
        self.left_pressed = false;
        self.right_pressed = false;
        self.home_pressed = false;
        self.end_pressed = false;
        self.tab_pressed = false;
        self.enter_pressed = false;
    }
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum GameKey {
    Left,
    Right,
    Up,
    Down,
    Jump,
}
