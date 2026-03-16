#[derive(Debug, Clone, Copy)]
pub(crate) enum Key {
    ZERO,
    ONE,
    TWO,
    THREE,
    FOUR,
    FIVE,
    SIX,
    SEVEN,
    EIGHT,
    NINE,
    A,
    B,
    C,
    D,
    E,
    F,
}

#[derive(Debug)]
pub(crate) struct Keyboard {}

impl Keyboard {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn is_down(k: Key) -> bool {
        todo!()
    }

    pub(crate) fn is_up(k: Key) -> bool {
        todo!()
    }

    pub(crate) fn next_key() -> Key {
        todo!()
    }
}
