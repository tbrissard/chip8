use crate::app::Action;

mod crossterm;

pub(crate) use crossterm::{CrosstermInputManager, KEYBINDS};

pub(crate) trait InputManager {
    type Err: std::error::Error + Send + Sync + 'static;

    /// Poll available events and translate them into [Action]
    /// Non-blocking
    fn poll_events() -> std::result::Result<Vec<Action>, Self::Err>;

    /// Get next event
    /// Blocking
    fn read_event() -> std::result::Result<Option<Action>, Self::Err>;
}
