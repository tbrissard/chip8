use std::time::{Duration, Instant};

pub(crate) use crate::keyboard::ch8key::{Ch8Key, KeyError};

mod ch8key;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeyState {
    #[default]
    Up,
    Down(Instant),
}

impl Default for Ch8Keyboard {
    fn default() -> Self {
        Self {
            states: [KeyState::Up; 16],
        }
    }
}

#[derive(Debug)]
pub(crate) struct Ch8Keyboard {
    states: [KeyState; 16],
}

impl Ch8Keyboard {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn is_down(&self, k: Ch8Key) -> bool {
        matches!(self[k], KeyState::Down(_))
    }

    pub(super) fn is_up(&self, k: Ch8Key) -> bool {
        self[k] == KeyState::Up
    }

    pub(crate) fn press_key(&mut self, key: Ch8Key) {
        self[key] = KeyState::Down(Instant::now())
    }

    pub(crate) fn release_key(&mut self, key: Ch8Key) {
        self[key] = KeyState::Up
    }

    /// Simulate the release of keys that have been pressed
    pub(super) fn release_keys(&mut self) {
        const PRESS_DURATION: Duration = Duration::from_millis(200);

        for k in Ch8Key::VARIANTS {
            if let KeyState::Down(instant) = self[k]
                && instant.elapsed() > PRESS_DURATION
            {
                self.release_key(k);
            }
        }
    }
}

impl std::ops::Index<Ch8Key> for Ch8Keyboard {
    type Output = KeyState;

    fn index(&self, index: Ch8Key) -> &Self::Output {
        &self.states[index as usize]
    }
}

impl std::ops::IndexMut<Ch8Key> for Ch8Keyboard {
    fn index_mut(&mut self, index: Ch8Key) -> &mut Self::Output {
        &mut self.states[index as usize]
    }
}
