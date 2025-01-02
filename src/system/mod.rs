use crate::state::State;

pub mod ui;

pub trait System {
    fn execute(&self, state: &mut State);
}
