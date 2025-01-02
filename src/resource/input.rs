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
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum CursorState {
    Idle,
    Hidden,
}
