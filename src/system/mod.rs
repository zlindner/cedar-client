use crate::{resource::input::CursorState, state::State};

pub trait System {
    fn execute(&self, state: &mut State);
}

#[derive(Default)]
pub struct CursorSystem;

impl System for CursorSystem {
    fn execute(&self, state: &mut State) {
        let mut cursor = state.cursor();

        if cursor.should_hide() {
            cursor.set_state(CursorState::Hidden);
            return;
        }

        // TODO: we need to check what the cursor is currently hovering over to determine the
        // correct state.
        cursor.set_state(CursorState::Idle);
    }
}
